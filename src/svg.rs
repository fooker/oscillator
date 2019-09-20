use std::path::Path;

use failure::Error;
use svg::node::element::path::{Command, Data, Position};
use svg::parser::Event;

use crate::model::{Coordinate, PathSegment};

fn coords(size: (f32, f32), x: f32, y: f32) -> Coordinate {
    let x = ((x / size.0 - 0.5) * std::u16::MAX as f32) as i16;
    let y = ((y / size.1 - 0.5) * std::u16::MAX as f32) as i16;

    return Coordinate { x, y };
}

/// Transforms maybe-relative coordinates into absolute ones
fn reladd(cp: Coordinate, pos: &Position, c: Coordinate) -> Coordinate {
    return match pos {
        Position::Absolute => c,
        Position::Relative => cp + c,
    };
}

pub fn parse_file(path: impl AsRef<Path>, size: (f32, f32)) -> Result<Vec<PathSegment>, Error> {
    let mut result = Vec::new();
    for event in svg::open(path.as_ref())? {
        match event {
            Event::Tag("path", _, attr) => {
                log::trace!("Path: {:?}", attr["d"]);
                let data = Data::parse(&attr["d"])?;

                // The first move in each path is always absolute (regardless of it's real type)
                // which is the same as resetting the current position to zero
                let mut ip = Coordinate::zero(); // The initial position of each sub-path
                let mut cp = Coordinate::zero(); // The current position after each command

                for command in data.iter() {
                    log::trace!("Command: {:?}", command);

                    match command {
                        Command::Move(pos, params) => {
                            cp = reladd(cp, pos, coords(size, params[0], params[1]));
                            ip = cp;

                            // The move command can have additional parameters which are handled as line commands
                            // The line commands are relative if the move is relative and vice versa
                            for i in (2..params.len()).step_by(2) {
                                let ep = reladd(cp, pos, coords(size, params[i + 0], params[i + 1]));
                                result.push(PathSegment::line(cp, ep));
                                cp = ep;
                            }
                        }

                        Command::Close => {
                            if cp != ip {
                                result.push(PathSegment::line(cp, ip));
                            }
                            cp = ip;
                        }

                        Command::Line(pos, params) => {
                            for i in (0..params.len()).step_by(2) {
                                let ep = reladd(cp, pos, coords(size, params[i + 0], params[i + 1]));
                                result.push(PathSegment::line(cp, ep));
                                cp = ep;
                            }
                        }
                        Command::HorizontalLine(pos, params) => {
                            for i in (0..params.len()).step_by(1) {
                                let ep = reladd(cp, pos, coords(size, params[i], 0f32));
                                result.push(PathSegment::line(cp, ep));
                                cp = ep;
                            }
                        }
                        Command::VerticalLine(pos, params) => {
                            for i in (0..params.len()).step_by(1) {
                                let ep = reladd(cp, pos, coords(size, 0f32, params[i]));
                                result.push(PathSegment::line(cp, ep));
                                cp = ep;
                            }
                        }

                        _ => {
                            log::warn!("Unsupported path command: {:?}", command);
                        }
                    }
                }
            }
            Event::Tag(name, kind, attr) => {
                log::trace!("Tag: {}({:?}) {:?}", name, kind, attr);
            }
            _ => {}
        }
    }

    return Ok(result);
}
