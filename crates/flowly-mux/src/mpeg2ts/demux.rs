use flowly_service::Service;
use mpeg2ts_reader::{
    demultiplex::{
        Demultiplex, DemuxContext, FilterChangeset, FilterRequest, NullPacketFilter, PacketFilter,
        PatPacketFilter, PmtPacketFilter,
    },
    packet::{Packet, Pid},
    pes::{self, Timestamp},
    psi, StreamType,
};

use bytes::{Bytes, BytesMut};
use futures::{Stream, TryStreamExt};

use flowly_core::{Codec, Frame};
use std::collections::HashMap;
use std::{collections::VecDeque, pin::pin};

const DEFAULT_CAPACITY: usize = 188 * 1024;

struct FlowlyDemuxContext {
    key_frame_flag: bool,
    changeset: FilterChangeset<FlowlyFilterSwitch>,
    consumers: HashMap<Pid, VecDeque<Mpeg2TsFrame>>,
}

impl FlowlyDemuxContext {
    pub fn new() -> Self {
        FlowlyDemuxContext {
            changeset: FilterChangeset::default(),
            consumers: HashMap::new(),
            key_frame_flag: false,
        }
    }
}

impl DemuxContext for FlowlyDemuxContext {
    type F = FlowlyFilterSwitch;

    fn filter_changeset(&mut self) -> &mut FilterChangeset<Self::F> {
        &mut self.changeset
    }

    fn construct(&mut self, req: FilterRequest<'_, '_>) -> Self::F {
        println!("{:#?}", req);

        match req {
            // The 'Program Association Table' is is always on PID 0.  We just use the standard
            // handling here, but an application could insert its own logic if required,
            FilterRequest::ByPid(Pid::PAT) => FlowlyFilterSwitch::Pat(PatPacketFilter::default()),

            // 'Stuffing' data on PID 0x1fff may be used to pad-out parts of the transport stream
            // so that it has constant overall bitrate.  This causes it to be ignored if present.
            FilterRequest::ByPid(Pid::STUFFING) => {
                FlowlyFilterSwitch::Null(NullPacketFilter::default())
            }

            // Some Transport Streams will contain data on 'well known' PIDs, which are not
            // announced in PAT / PMT metadata.  This application does not process any of these
            // well known PIDs, so we register NullPacketFiltet such that they will be ignored
            FilterRequest::ByPid(_) => FlowlyFilterSwitch::Null(NullPacketFilter::default()),

            // This match-arm installs our application-specific handling for each H264 stream
            // discovered within the transport stream,
            FilterRequest::ByStream {
                stream_type: StreamType::H265,
                pmt,
                stream_info,
                ..
            } => PtsFlowlyElementaryStreamConsumer::construct(pmt, stream_info),

            // We need to have a match-arm to specify how to handle any other StreamType values
            // that might be present; we answer with NullPacketFilter so that anything other than
            // H264 (handled above) is ignored,
            FilterRequest::ByStream { .. } => FlowlyFilterSwitch::Null(NullPacketFilter::default()),

            // The 'Program Map Table' defines the sub-streams for a particular program within the
            // Transport Stream (it is common for Transport Streams to contain only one program).
            // We just use the standard handling here, but an application could insert its own
            // logic if required,
            FilterRequest::Pmt {
                pid,
                program_number,
            } => FlowlyFilterSwitch::Pmt(PmtPacketFilter::new(pid, program_number)),

            // Ignore 'Network Information Table', if present,
            FilterRequest::Nit { .. } => FlowlyFilterSwitch::Null(NullPacketFilter::default()),
        }
    }
}

enum FlowlyFilterSwitch {
    Pes(pes::PesPacketFilter<FlowlyDemuxContext, PtsFlowlyElementaryStreamConsumer>),
    Pat(PatPacketFilter<FlowlyDemuxContext>),
    Pmt(PmtPacketFilter<FlowlyDemuxContext>),
    Null(NullPacketFilter<FlowlyDemuxContext>),
}

impl PacketFilter for FlowlyFilterSwitch {
    type Ctx = FlowlyDemuxContext;

    #[inline(always)]
    fn consume(&mut self, ctx: &mut FlowlyDemuxContext, pk: &Packet<'_>) {
        ctx.key_frame_flag = pk
            .adaptation_field()
            .map(|x| x.random_access_indicator())
            .unwrap_or(false);

        match self {
            &mut FlowlyFilterSwitch::Pes(ref mut f) => f.consume(ctx, pk),
            &mut FlowlyFilterSwitch::Pat(ref mut f) => f.consume(ctx, pk),
            &mut FlowlyFilterSwitch::Pmt(ref mut f) => f.consume(ctx, pk),
            &mut FlowlyFilterSwitch::Null(ref mut f) => f.consume(ctx, pk),
        }
    }
}

// Implement the ElementaryStreamConsumer to just dump and PTS/DTS timestamps to stdout
pub struct PtsFlowlyElementaryStreamConsumer {
    pid: Pid,
    pts: Timestamp,
    dts: Option<Timestamp>,
    counter: u64,
    codec: Codec,
    buffer: BytesMut,
    is_keyframe: bool,
}

impl PtsFlowlyElementaryStreamConsumer {
    fn construct(
        _pmt_sect: &psi::pmt::PmtSection,
        stream_info: &psi::pmt::StreamInfo,
    ) -> FlowlyFilterSwitch {
        let filter = pes::PesPacketFilter::new(PtsFlowlyElementaryStreamConsumer {
            pid: stream_info.elementary_pid(),
            buffer: BytesMut::with_capacity(DEFAULT_CAPACITY),
            pts: Timestamp::MAX,
            dts: None,
            counter: 0u64,
            codec: match stream_info.stream_type() {
                StreamType::H264 => Codec::H264,
                StreamType::H265 => Codec::H265,
                _ => Codec::UNK,
            },
            is_keyframe: false,
        });
        FlowlyFilterSwitch::Pes(filter)
    }
}

impl pes::ElementaryStreamConsumer<FlowlyDemuxContext> for PtsFlowlyElementaryStreamConsumer {
    fn start_stream(&mut self, _ctx: &mut FlowlyDemuxContext) {}
    fn begin_packet(&mut self, ctx: &mut FlowlyDemuxContext, header: pes::PesHeader) {
        match header.contents() {
            pes::PesContents::Parsed(Some(parsed)) => {
                match parsed.pts_dts() {
                    Ok(pes::PtsDts::PtsOnly(Ok(pts))) => {
                        self.pts = pts;
                        self.dts = None;
                    }

                    Ok(pes::PtsDts::Both {
                        pts: Ok(pts),
                        dts: Ok(dts),
                    }) => {
                        self.pts = pts;
                        self.dts = Some(dts);
                    }

                    _ => (),
                }

                self.is_keyframe = ctx.key_frame_flag;
                self.buffer.clear();
                self.buffer.extend_from_slice(parsed.payload());
            }
            pes::PesContents::Parsed(None) => (),
            pes::PesContents::Payload(payload) => {
                self.buffer.extend_from_slice(payload);
            }
        }
    }

    fn continue_packet(&mut self, _ctx: &mut FlowlyDemuxContext, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    fn end_packet(&mut self, ctx: &mut FlowlyDemuxContext) {
        let units = self.buffer.split().freeze();

        let params = match self.codec {
            Codec::H264 => split_params_h264(&units),
            Codec::H265 => split_params_h265(&units),
            _ => None,
        };

        ctx.consumers
            .entry(self.pid)
            .or_default()
            .push_back(Mpeg2TsFrame {
                pts: (self.pts.value() * 100) / 9,
                dts: self.dts.map(|dts| (dts.value() * 100) / 9),
                params,
                units,
                is_keyframe: self.is_keyframe,
                seq: self.counter,
                codec: self.codec,
            });
        self.counter += 1;
    }
    fn continuity_error(&mut self, _ctx: &mut FlowlyDemuxContext) {}
}

pub enum Void {}

#[derive(Debug, thiserror::Error)]
pub enum Error<E = Void> {
    #[error("Demux Error: {0:?}")]
    Mpeg2TsError(mpeg2ts_reader::demultiplex::DemuxError),

    #[error(transparent)]
    Other(E),
}

#[derive(Debug, Clone)]
pub struct Mpeg2TsFrame {
    seq: u64,
    pts: u64,
    dts: Option<u64>,
    codec: Codec,
    params: Option<Bytes>,
    units: Bytes,
    is_keyframe: bool,
}

impl Frame for Mpeg2TsFrame {
    fn seq(&self) -> u64 {
        self.seq
    }

    fn pts(&self) -> u64 {
        self.pts
    }

    fn timestamp(&self) -> Option<u64> {
        None
    }

    fn dts(&self) -> Option<u64> {
        self.dts
    }

    fn codec(&self) -> Codec {
        self.codec
    }

    fn flags(&self) -> flowly_core::FrameFlags {
        let mut flags = flowly_core::FrameFlags::empty();
        flags.set(flowly_core::FrameFlags::KEY_FRAME, self.is_keyframe);
        flags.set(flowly_core::FrameFlags::HAS_DTS, self.dts.is_some());
        flags.set(flowly_core::FrameFlags::HAS_PARAMS, self.params.is_some());
        flags
    }

    fn params(&self) -> impl Iterator<Item = Bytes> {
        self.params
            .iter()
            .map(|p| annexb_iter(p).map(|(_, from, to)| p.slice(from..to)))
            .flatten()
    }

    fn units(&self) -> impl Iterator<Item = Bytes> {
        annexb_iter(&self.units).map(|(_, from, to)| self.units.slice(from..to))
    }
}

pub struct Mpeg2TsDemux {
    ctx: FlowlyDemuxContext,
}

impl Mpeg2TsDemux {
    pub fn new() -> Self {
        Self {
            ctx: FlowlyDemuxContext::new(),
        }
    }
}

impl<F, E> Service<Result<F, E>> for Mpeg2TsDemux
where
    F: AsRef<[u8]> + Send + 'static,
    E: Send + 'static,
{
    type Out = Result<Mpeg2TsFrame, Error<E>>;

    fn handle(
        mut self,
        input: impl Stream<Item = Result<F, E>> + Send,
    ) -> impl Stream<Item = Self::Out> + Send {
        async_stream::try_stream! {
            let mut demux = Demultiplex::new(&mut self.ctx);
            let mut stream = pin!(input);

            while let Some(frame) = stream.try_next().await.map_err(Error::Other)? {
                demux.push(&mut self.ctx, &frame.as_ref());

                while let Some(pkt) = self.ctx.consumers.values_mut().next().unwrap().pop_front() {
                    yield pkt;
                }
            }
        }
    }
}

fn annexb_iter(data: &Bytes) -> impl Iterator<Item = (usize, usize, usize)> {
    let mut prev = 0;
    let mut prev1 = 0;
    (0..data.len()).filter_map(move |idx| {
        if data[idx + 1..].starts_with(&[0u8, 0, 1]) {
            let (from, from1) = (prev, prev1);
            prev = if data[idx] == 0 { idx } else { idx + 1 };
            prev1 = idx + 4;

            (!data[from1..prev].is_empty()).then_some((from, from1, prev))
        } else if idx == 0 {
            if data.starts_with(&[0u8, 0, 1]) {
                prev1 = 3;
            } else if data.starts_with(&[0u8, 0, 0, 1]) {
                prev1 = 4;
            }
            None
        } else if idx == data.len().saturating_sub(3) {
            (!data[prev1..].is_empty()).then_some((prev, prev1, data.len()))
        } else {
            None
        }
    })
}

fn split_params_h264(data: &Bytes) -> Option<Bytes> {
    let mut ps_range = None;

    for (wt, from, to) in annexb_iter(&data.clone()) {
        match data[from] >> 1 {
            7 => ps_range = Some((wt, to)), // SPS
            8 => ps_range = Some((ps_range.map(|(f, _)| f).unwrap_or(wt), to)), // PPS
            _ => (),
        }
    }

    ps_range.map(|(from, to)| data.slice(from..to))
}

fn split_params_h265(data: &Bytes) -> Option<Bytes> {
    let mut ps_range = None;

    for (wt, from, to) in annexb_iter(&data.clone()) {
        match data[from] >> 1 {
            32 => ps_range = Some((wt, to)), // VPS
            33 => ps_range = Some((ps_range.map(|(f, _)| f).unwrap_or(wt), to)), // SPS
            34 => ps_range = Some((ps_range.map(|(f, _)| f).unwrap_or(wt), to)), // PPS
            _ => (),
        }
    }

    ps_range.map(|(from, to)| data.slice(from..to))
}
