use std::io::{Read, Write};

use anyhow::bail;
use dwd_bufr_tables::{DWD_BUFR_TABLE_B, DWD_BUFR_TABLE_D};
use dwd_gts::GtsHeader;
use tinybufr::{DataEvent, DataReader, DataSpec, HeaderSections, Tables};

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    let _ = args.next();
    let Some(input) = args.next() else {
        bail!("Missing input");
    };
    let output = args.next();

    let reader = std::fs::File::open(input)?;
    let writer: Box<dyn Write> = if let Some(out) = output {
        Box::new(
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(out)?,
        )
    } else {
        Box::new(std::io::stdout().lock())
    };
    let mut dumper = Dumper::new(writer);
    handle_file(reader, &mut dumper)?;

    Ok(())
}

struct Dumper<W: Write> {
    indent: usize,
    writer: W,
}

impl<W: Write> Dumper<W> {
    pub fn new(writer: W) -> Self {
        Self { indent: 0, writer }
    }

    pub fn writer(&mut self) -> std::io::Result<&mut W> {
        for _ in 0..self.indent {
            self.writer.write_all(b"  ")?;
        }
        Ok(&mut self.writer)
    }

    pub fn indent(&mut self) {
        self.indent += 1;
    }

    pub fn dedent(&mut self) {
        self.indent -= 1;
    }
}

fn dump<R: Read>(
    mut bounded: R,
    tables: &Tables,
    dumper: &mut Dumper<impl Write>,
) -> anyhow::Result<R> {
    let header = HeaderSections::read(&mut bounded).unwrap();
    let data_spec =
        DataSpec::from_data_description(&header.data_description_section, &tables).unwrap();
    let mut data_reader = DataReader::new(bounded, &data_spec).unwrap();

    loop {
        match data_reader.read_event() {
            Ok(DataEvent::SubsetStart(id)) => {
                writeln!(dumper.writer()?, "- {id}:")?;
                dumper.indent();
            }
            Ok(DataEvent::SubsetEnd) => dumper.dedent(),
            Ok(DataEvent::ReplicationStart { idx, count }) => {
                writeln!(dumper.writer()?, "- replicate_{idx}({count}):")?;
                dumper.indent();
            }
            Ok(DataEvent::ReplicationEnd) => dumper.dedent(),
            Ok(DataEvent::ReplicationItemStart) => {
                writeln!(dumper.writer()?, "-")?;
                dumper.indent();
            }
            Ok(DataEvent::ReplicationItemEnd) => dumper.dedent(),
            Ok(DataEvent::SequenceStart { xy, .. }) => {
                if let Some(xd) = tables.table_d.get(&xy) {
                    writeln!(dumper.writer()?, "# {}", xd.title)?;
                }
                writeln!(dumper.writer()?, "- 3 {} {}:", xy.x, xy.y)?;
                dumper.indent();
            }
            Ok(DataEvent::SequenceEnd) => dumper.dedent(),
            Ok(DataEvent::OperatorHandled { .. }) => (),
            Ok(DataEvent::Data { xy, value, .. }) => {
                let w = dumper.writer()?;
                write!(w, "- 0 {} {}: {value:?}", xy.x, xy.y)?;
                if let Some(xd) = tables.table_b.get(&xy) {
                    write!(w, " # [{}] {}", xd.unit, xd.element_name)?;
                }
                writeln!(w)?;
            }

            Ok(DataEvent::Eof) => break,
            Err(e) => return Err(e.into()),
            _ => unimplemented!(),
        }
    }
    Ok(data_reader.into_inner())
}

fn handle_file(mut r: impl Read, dumper: &mut Dumper<impl Write>) -> anyhow::Result<()> {
    let tables = make_tables();

    loop {
        // 8 bytes ASCII message length
        let mut gts_bytes = [0u8; 8];
        r.read_exact(&mut gts_bytes)?;
        let Some(length) = atoi::atoi::<u64>(&gts_bytes).filter(|x| *x > 0) else {
            break;
        };
        // skip two '0'
        r.read_exact(&mut gts_bytes[..2])?;

        let mut bounded = r.by_ref().take(length);
        let mut gts_header_bytes = [0; 35];
        bounded.read_exact(&mut gts_header_bytes[..31])?;
        if &gts_header_bytes[28..31] != b"\r\r\n" {
            bounded.read_exact(&mut gts_header_bytes[31..])?;
            if &gts_header_bytes[32..] != b"\r\r\n" {
                bail!("invalid GTC header end");
            }
        }
        let gts_header = GtsHeader::read(&gts_header_bytes)?;
        println!("{} {}", gts_header.product_id, gts_header.source);
        if length == 31 + 7 {
            let mut nil_end = [0; 7];
            bounded.read_exact(&mut nil_end)?;
            if &nil_end != b"NIL\r\r\n\x03" {
                bail!("Invalid nil message");
            }
            continue;
        }

        writeln!(
            dumper.writer()?,
            "- {} {}:",
            gts_header.product_id,
            gts_header.source
        )?;
        dumper.indent();

        let mut bounded = dump(bounded, &tables, dumper)?;
        let mut footer = [0; 8];
        bounded.read_exact(&mut footer)?;
        if &footer != b"7777\r\r\n\x03" {
            bail!("invalid BUFR end or GTC end");
        }

        dumper.dedent();
    }

    Ok(())
}

fn make_tables() -> Tables {
    let mut tables = Tables::default();
    for it in &DWD_BUFR_TABLE_B {
        tables.table_b.insert(it.xy, it);
    }
    for it in &DWD_BUFR_TABLE_D {
        tables.table_d.insert(it.xy, it);
    }
    tables
}
