use crate::bit_stream::BitStream;
use std::cmp::max;
use std::f64::consts::PI;
use crate::error::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    pub yaw: Option<f64>,
    pub pitch: Option<f64>,
    pub roll: Option<f64>,
    pub mx: f64,
    pub my: f64,
    pub mz: f64,
    pub freelook: bool,
    pub triggers: [bool; 6],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub moves: [Option<Move>; 2],
    pub delta: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub mission: String,
    pub frames: Vec<Frame>,
}

impl Move {
    pub fn from_stream(bs: &mut BitStream) -> Result<Move> {
        let yaw = bs.read_optional(|bs| Move::read_angle(bs))?;
        let pitch = bs.read_optional(|bs| Move::read_angle(bs))?;
        let roll = bs.read_optional(|bs| Move::read_angle(bs))?;
        let mx = bs.read_scaled_f64_bits(6, 1f64 / 16f64, -1.0f64)?;
        let my = bs.read_scaled_f64_bits(6, 1f64 / 16f64, -1.0f64)?;
        let mz = bs.read_scaled_f64_bits(6, 1f64 / 16f64, -1.0f64)?;
        let freelook = bs.read_bool()?;
        let mut triggers = [false; 6];
        for i in 0..6 {
            triggers[i] = bs.read_bool()?;
        }
        Ok(Move {
            yaw,
            pitch,
            roll,
            mx,
            my,
            mz,
            freelook,
            triggers,
        })
    }

    pub fn into_stream(self, bs: &mut BitStream) -> Result<()> {
        bs.write_optional(self.yaw, |bs, angle| Move::write_angle(bs, angle))?;
        bs.write_optional(self.pitch, |bs, angle| Move::write_angle(bs, angle))?;
        bs.write_optional(self.roll, |bs, angle| Move::write_angle(bs, angle))?;
        bs.write_scaled_f64_bits(self.mx, 6, 1f64 / 16f64, -1.0f64)?;
        bs.write_scaled_f64_bits(self.my, 6, 1f64 / 16f64, -1.0f64)?;
        bs.write_scaled_f64_bits(self.mz, 6, 1f64 / 16f64, -1.0f64)?;
        bs.write_bool(self.freelook)?;
        for i in 0..6 {
            bs.write_bool(self.triggers[i])?;
        }
        Ok(())
    }

    fn read_angle(bs: &mut BitStream) -> Result<f64> {
        // Torque scales these from [-pi, pi] -> [0, 2^16]
        let angle = bs.read_scaled_f64_bits(16, PI / 32768f64, 0f64)?;
        if angle >= PI {
            Ok(angle - 2f64 * PI)
        } else {
            Ok(angle)
        }
    }

    fn write_angle(bs: &mut BitStream, mut angle: f64) -> Result<()> {
        // Torque scales these from [-pi, pi] -> [0, 2^16]
        if angle < 0f64 {
            angle += 2f64 * PI;
        }
        bs.write_scaled_f64_bits(angle, 16, PI / 32768f64, 0f64)
    }
}

impl Frame {
    pub fn from_stream(bs: &mut BitStream) -> Result<Frame> {
        let move0 = bs.read_optional(|bs| Move::from_stream(bs))?;
        let move1 = bs.read_optional(|bs| Move::from_stream(bs))?;
        let delta = bs.read_bits_u16(10)?;
        Ok(Frame {
            moves: [move0, move1],
            delta,
        })
    }

    pub fn into_stream(self, bs: &mut BitStream) -> Result<()> {
        let [m1, m2] = self.moves;
        bs.write_optional(m1, |bs, mv| mv.into_stream(bs))?;
        bs.write_optional(m2, |bs, mv| mv.into_stream(bs))?;
        bs.write_bits_u16(self.delta, 10)
    }

    pub fn has_move(&self) -> bool {
        self.moves[0].is_some() || self.moves[1].is_some()
    }
}

impl Recording {
    pub fn from_stream(bs: &mut BitStream) -> Result<Recording> {
        let mission = bs.read_string()?;
        let mut frames = vec![];

        while !bs.eof() {
            let length = bs.read_u8()?;
            if length == 0 {
                break;
            }

            let mut data = Vec::with_capacity(length as usize);
            for _ in 0..length {
                if bs.eof() {
                    return Ok(Recording { mission, frames });
                }
                data.push(bs.read_u8()?);
            }

            let mut inner_stream = BitStream::new(data);
            frames.push(Frame::from_stream(&mut inner_stream)?);
        }

        Ok(Recording { mission, frames })
    }

    pub fn into_stream(self, bs: &mut BitStream) -> Result<()> {
        bs.write_string(self.mission)?;

        for frame in self.frames {
            let mut inner_stream = BitStream::new(vec![]);
            frame.into_stream(&mut inner_stream)?;

            let bytes = inner_stream.bytes();
            let len = max(bytes.len(), 4);
            let extra = len - bytes.len();
            bs.write_u8(len as u8)?;

            for byte in bytes {
                bs.write_u8(byte)?;
            }
            for _ in 0..extra {
                bs.write_u8(0u8)?;
            }
        }

        Ok(())
    }
}
