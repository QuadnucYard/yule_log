use crate::errors::ULogError;
use crate::message_buf::MessageBuf;

pub trait ParseFromBuf: Sized {
    fn parse_from_buf(buf: &mut MessageBuf) -> Result<Self, ULogError>;
}

impl ParseFromBuf for u8 {
    fn parse_from_buf(buf: &mut MessageBuf) -> Result<Self, ULogError> {
        buf.take_u8()
    }
}
impl ParseFromBuf for u16 {
    fn parse_from_buf(buf: &mut MessageBuf) -> Result<Self, ULogError> {
        buf.take_u16()
    }
}
impl ParseFromBuf for u32 {
    fn parse_from_buf(buf: &mut MessageBuf) -> Result<Self, ULogError> {
        buf.take_u32()
    }
}
impl ParseFromBuf for u64 {
    fn parse_from_buf(buf: &mut MessageBuf) -> Result<Self, ULogError> {
        buf.take_u64()
    }
}
impl ParseFromBuf for i8 {
    fn parse_from_buf(buf: &mut MessageBuf) -> Result<Self, ULogError> {
        buf.take_i8()
    }
}
impl ParseFromBuf for i16 {
    fn parse_from_buf(buf: &mut MessageBuf) -> Result<Self, ULogError> {
        buf.take_i16()
    }
}
impl ParseFromBuf for i32 {
    fn parse_from_buf(buf: &mut MessageBuf) -> Result<Self, ULogError> {
        buf.take_i32()
    }
}
impl ParseFromBuf for i64 {
    fn parse_from_buf(buf: &mut MessageBuf) -> Result<Self, ULogError> {
        buf.take_i64()
    }
}
impl ParseFromBuf for f32 {
    fn parse_from_buf(buf: &mut MessageBuf) -> Result<Self, ULogError> {
        buf.take_f32()
    }
}
impl ParseFromBuf for f64 {
    fn parse_from_buf(buf: &mut MessageBuf) -> Result<Self, ULogError> {
        buf.take_f64()
    }
}
impl ParseFromBuf for bool {
    fn parse_from_buf(buf: &mut MessageBuf) -> Result<Self, ULogError> {
        Ok(buf.take_u8()? != 0)
    }
}
impl ParseFromBuf for char {
    fn parse_from_buf(buf: &mut MessageBuf) -> Result<Self, ULogError> {
        Ok(buf.take_u8()? as char)
    }
}

pub fn parse_data_field<T: ParseFromBuf>(message_buf: &mut MessageBuf) -> Result<T, ULogError> {
    T::parse_from_buf(message_buf)
}

pub fn parse_array<T, F>(
    array_size: usize,
    message_buf: &mut MessageBuf,
    mut parse_element: F,
) -> Result<Vec<T>, ULogError>
where
    F: FnMut(&mut MessageBuf) -> Result<T, ULogError>,
{
    let mut vec: Vec<T> = Vec::with_capacity(array_size);

    // SAFETY: We are manually initializing the elements of the vector
    unsafe {
        let dst_ptr = vec.as_mut_ptr();
        for i in 0..array_size {
            dst_ptr.add(i).write(parse_element(message_buf)?);
        }
        vec.set_len(array_size);
    }

    Ok(vec)
}

/// Parses an array of type T from the message buffer.
///
/// `T` requires to be primitive type.
pub fn parse_typed_array<T>(
    array_size: usize,
    message_buf: &mut MessageBuf,
) -> Result<Vec<T>, ULogError>
where
    T: ParseFromBuf + 'static,
{
    // Fast path: direct memory cast for primitive types on little-endian systems
    // This avoids the overhead of parsing each element individually
    #[cfg(target_endian = "little")]
    if std::mem::size_of::<T>() > 0 {
        let byte_size = array_size * std::mem::size_of::<T>();

        // Check we have enough bytes
        if message_buf.remaining_bytes().len() >= byte_size {
            let bytes = message_buf.advance(byte_size)?;

            // Check alignment
            if (bytes.as_ptr() as usize).is_multiple_of(std::mem::align_of::<T>()) {
                // SAFETY: This is safe because:
                // 1. T is a primitive type with known alignment
                // 2. ULog format uses little-endian (same as this target architecture)
                // 3. bytes.len() is exactly array_size * size_of::<T>()
                // 4. The pointer is properly aligned (checked above)
                // 5. We properly allocate and initialize the Vec before copying
                unsafe {
                    let src_ptr = bytes.as_ptr() as *const T;
                    let mut vec = Vec::with_capacity(array_size);
                    let dst_ptr = vec.as_mut_ptr();
                    std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, array_size);
                    vec.set_len(array_size);
                    return Ok(vec);
                }
            }

            // SAFETY: This is safe because:
            // 1. T is a primitive numeric type
            // 2. ULog format uses little-endian (same as this target architecture)
            // 3. bytes.len() is exactly array_size * size_of::<T>()
            // 4. We use read_unaligned to handle potentially unaligned data
            // 5. We properly allocate and initialize the Vec
            unsafe {
                let src_ptr = bytes.as_ptr() as *const T;
                let mut vec: Vec<T> = Vec::with_capacity(array_size);
                let dst_ptr = vec.as_mut_ptr();

                // Use read_unaligned for each element to handle unaligned data
                for i in 0..array_size {
                    dst_ptr.add(i).write(src_ptr.add(i).read_unaligned());
                }

                // Now we can safely set the length since all elements are initialized
                vec.set_len(array_size);
                return Ok(vec);
            }
        }
    }

    // Slow path: parse element by element
    // Used on big-endian systems or for non-primitive types
    let mut array = Vec::with_capacity(array_size);
    for _ in 0..array_size {
        array.push(parse_data_field(message_buf)?);
    }
    Ok(array)
}
