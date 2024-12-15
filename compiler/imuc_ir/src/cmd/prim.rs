use crate::prelude::*;
use imuc_ast::prim::*;

impl Rw for Prim {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let begin = input.read_char()?;
        let prim = match begin {
            'u' => Self::Unit,
            'b' => {
                let value = input.read_until(' ')?.parse()?;
                Self::Integer(Integer::I8(value))
            }
            'd' => {
                let value = input.read_until(' ')?.parse()?;
                Self::Integer(Integer::I16(value))
            }
            'q' => {
                let value = input.read_until(' ')?.parse()?;
                Self::Integer(Integer::I32(value))
            }
            'o' => {
                let value = input.read_until(' ')?.parse()?;
                Self::Integer(Integer::I64(value))
            }
            'f' => {
                let value = input.read_until(' ')?.parse()?;
                Self::Float(Float::F32(value))
            }
            'l' => {
                let value = input.read_until(' ')?.parse()?;
                Self::Float(Float::F64(value))
            }
            '\"' => {
                let mut value = String::new();
                let mut escape = false;
                loop {
                    let ch = input.read_char()?;
                    if escape {
                        escape = false;
                    } else {
                        match ch {
                            '\\' => {
                                escape = true;
                            }
                            '\"' => {
                                break;
                            }
                            _ => {}
                        }
                        value.push(ch);
                    }
                }
                let value = unescape::unescape(&value)
                    .ok_or_else(|| errors::IrError::UnknownEscape(value))?;
                Self::String(value)
            }
            _ => {
                return Err(errors::IrError::NoSuchPrimitive(begin).into());
            }
        };
        Ok(prim)
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        match self {
            Prim::Unit => {
                write!(output, "u")?;
            }
            Prim::Integer(integer) => match integer {
                Integer::I8(value) => {
                    write!(output, "b{}", value)?;
                }
                Integer::I16(value) => {
                    write!(output, "d{}", value)?;
                }
                Integer::I32(value) => {
                    write!(output, "q{}", value)?;
                }
                Integer::I64(value) => {
                    write!(output, "o{}", value)?;
                }
            },
            Prim::Float(float) => match float {
                Float::F32(value) => {
                    write!(output, "f{:.6}", value)?;
                }
                Float::F64(value) => {
                    write!(output, ";{:.10}", value)?;
                }
            },
            Prim::String(value) => {
                write!(output, "{:?}", value)?;
            }
        }
        Ok(())
    }
}
