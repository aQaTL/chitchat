use actix_web::web::Bytes;
use futures_util::task::{Context, Poll};
use std::io::Read;
use tokio::macros::support::Pin;
use tokio::stream::Stream;

const TEMPLATE_PATH: &str = "src/raw.html";

pub struct PasteRenderer {
    pieces: Vec<Piece>,
    idx: usize,
}

#[derive(Debug)]
enum Piece {
    Template(Vec<u8>),
    Arg(Vec<u8>),
}

impl PasteRenderer {
    pub fn new(args: &[Vec<u8>]) -> std::io::Result<Self> {
        let mut template_data = std::fs::File::open(TEMPLATE_PATH)?;
        let mut buf = vec![0u8; 1024 * 16];

        let mut pieces = Vec::new();
        let mut args = args.iter();

        while let Ok(read_bytes) = template_data.read(&mut buf) {
            let buf = &buf[0..read_bytes];
            if buf.is_empty() {
                break;
            }

            let mut start = 0;
            let mut idx = 0;
            while idx < (buf.len() - 1) {
                if buf[idx] == b'\\' && buf[idx + 1] == b'!' {
                    if buf[idx + 1] == b'!' {
                        pieces.push(Piece::Template(buf[start..idx].to_vec()));
                        pieces.push(Piece::Arg(args.next().unwrap().clone()));
                        start = idx + 2;
                        idx = start + 1;
                    }
                } else {
                    idx += 1;
                }
            }
            pieces.push(Piece::Template(buf[start..=idx].to_vec()));
        }

        Ok(PasteRenderer { pieces, idx: 0 })
    }
}

impl Stream for PasteRenderer {
    type Item = Result<Bytes, ()>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.idx != self.pieces.len() {
            self.idx += 1;
            Poll::Ready(Some(Ok(Bytes::from(match &self.pieces[self.idx - 1] {
                Piece::Template(buf) => buf.clone(),
                Piece::Arg(buf) => format!(
                    "{}",
                    v_htmlescape::escape(std::str::from_utf8(buf).unwrap())
                )
                .into_bytes(),
            }))))
        } else {
            Poll::Ready(None)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        unimplemented!()
    }
}
