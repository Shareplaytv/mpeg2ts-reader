use mpeg2ts_reader::packet_filter_switch;
use mpeg2ts_reader::demux_context;
use mpeg2ts_reader::demultiplex;
use mpeg2ts_reader::pes;
use mpeg2ts_reader::psi;
use std::fs::File;
use std::io::Read;

packet_filter_switch! {
    NullFilterSwitch<NullDemuxContext> {
        Pat: demultiplex::PatPacketFilter<NullDemuxContext>,
        Pmt: demultiplex::PmtPacketFilter<NullDemuxContext>,
        Null: demultiplex::NullPacketFilter<NullDemuxContext>,
        NullPes: pes::PesPacketFilter<NullDemuxContext,NullElementaryStreamConsumer>,
    }
}
demux_context!(NullDemuxContext, NullFilterSwitch);
impl NullDemuxContext {
    fn do_construct(&mut self, req: demultiplex::FilterRequest<'_, '_>) -> NullFilterSwitch {
        match req {
            demultiplex::FilterRequest::ByPid(psi::pat::PAT_PID) => {
                NullFilterSwitch::Pat(demultiplex::PatPacketFilter::default())
            }
            demultiplex::FilterRequest::ByPid(_) => {
                NullFilterSwitch::Null(demultiplex::NullPacketFilter::default())
            }
            demultiplex::FilterRequest::ByStream {
                pmt, stream_info, ..
            } => NullElementaryStreamConsumer::construct(pmt, stream_info),
            demultiplex::FilterRequest::Pmt {
                pid,
                program_number,
            } => NullFilterSwitch::Pmt(demultiplex::PmtPacketFilter::new(pid, program_number)),
            demultiplex::FilterRequest::Nit { .. } => {
                NullFilterSwitch::Null(demultiplex::NullPacketFilter::default())
            }
        }
    }
}

pub struct NullElementaryStreamConsumer {}
impl NullElementaryStreamConsumer {
    fn construct(
        _pmt_sect: &psi::pmt::PmtSection,
        stream_info: &psi::pmt::StreamInfo,
    ) -> NullFilterSwitch {
        println!("stream info: {:?}", stream_info);
        let filter = pes::PesPacketFilter::new(NullElementaryStreamConsumer {});
        NullFilterSwitch::NullPes(filter)
    }
}
impl<Ctx> pes::ElementaryStreamConsumer<Ctx> for NullElementaryStreamConsumer {
    fn start_stream(&mut self, _ctx: &mut Ctx) {}
    fn begin_packet(&mut self, _ctx: &mut Ctx, header: pes::PesHeader) {
        if let pes::PesContents::Parsed(Some(content)) = header.contents() {
            match content.pts_dts() {
                Ok(pes::PtsDts::PtsOnly(Ok(ts))) => {
                    criterion::black_box(ts);
                }
                Ok(pes::PtsDts::Both { pts: Ok(ts), .. }) => {
                    criterion::black_box(ts);
                }
                _ => (),
            };
        }
    }
    fn continue_packet(&mut self, _ctx: &mut Ctx, _data: &[u8]) {}
    fn end_packet(&mut self, _ctx: &mut Ctx) {}
    fn continuity_error(&mut self, _ctx: &mut Ctx) {}
}

fn mpeg2ts_reader() {
    let mut f = File::open("586000000.ts")
        .expect("Test file missing");
    let l = f.metadata().unwrap().len() as usize;
    let size = l.min(188 * 200_000);
    let mut buf = vec![0; size];
    f.read(&mut buf[..]).unwrap();

    let mut ctx = NullDemuxContext::new();
    let mut demux = demultiplex::Demultiplex::new(&mut ctx);
    demux.push(&mut ctx, &buf[..]);
}

iai::main!(
    mpeg2ts_reader,
);
