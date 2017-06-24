#[macro_use]
extern crate quickcheck;
extern crate byteorder;
extern crate scroll;

use byteorder::*;
use scroll::*;
use scroll::ctx::str::*;
use scroll::ctx::bytes::*;

#[test]
fn test_str_pread() {
    let bytes: &[u8] = b"hello, world!\0some_other_things";
    assert_eq!(TryRead::try_read(bytes, StrCtx::Delimiter(NULL)).unwrap(),
               ("hello, world!", 13));
    assert!(bytes
                .pread_with::<&str>(0, StrCtx::Delimiter(RET))
                .is_err());

    let bytes: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    assert_eq!(TryRead::try_read(bytes, StrCtx::Length(15)).unwrap(),
               ("abcdefghijklmno", 15));
    assert_eq!(TryRead::try_read(bytes, StrCtx::Length(26)).unwrap(),
               ("abcdefghijklmnopqrstuvwxyz", 26));

    assert!(bytes.pread_with::<&str>(0, StrCtx::Length(26)).is_ok());
    assert!(bytes.pread_with::<&str>(0, StrCtx::Length(27)).is_err());
    assert!(bytes.pread_with::<&str>(27, StrCtx::Length(0)).is_err());
    assert!(bytes.pread_with::<&str>(26, StrCtx::Length(1)).is_err());
}

#[test]
fn test_str_gread() {
    let bytes: &[u8] = b"hello, world!\0some_other_things";
    let mut offset = 0;
    let s: &str = bytes
        .gread_with(&mut offset, StrCtx::Delimiter(NULL))
        .unwrap();
    assert_eq!(s, "hello, world!");
    assert_eq!(offset, 13);
}

#[test]
fn test_str_delimitor_until() {
    let bytes: &[u8] = b"hello, world!\0some_other_things";

    assert_eq!(TryRead::try_read(bytes, StrCtx::DelimiterUntil(NULL, 20)).unwrap(),
               ("hello, world!", 13));
    assert_eq!(TryRead::try_read(bytes, StrCtx::DelimiterUntil(NULL, 13)).unwrap(),
               ("hello, world!", 13));
    assert_eq!(TryRead::try_read(bytes, StrCtx::DelimiterUntil(NULL, 10)).unwrap(),
               ("hello, wor", 10));

    let bytes: &[u8] = b"hello, world!";
    assert!(bytes
                .pread_with::<&str>(0, StrCtx::DelimiterUntil(NULL, 20))
                .is_err());
    assert!(bytes
                .pread_with::<&str>(0, StrCtx::Delimiter(NULL))
                .is_err());
}

#[test]
fn test_str_pwrite() {
    let mut bytes = [0; 20];
    bytes.pwrite(0, "hello world!").unwrap();
    assert_eq!(&bytes[..12], b"hello world!" as &[u8]);

    let mut bytes = &mut [0; 10];
    assert!(bytes.pwrite(0, "hello world!").is_err());
}

#[test]
fn test_str_gwrite() {
    let mut bytes = [0; 20];
    let mut offset = 0;
    bytes.gwrite(&mut offset, "hello world!").unwrap();
    assert_eq!(offset, 12);
    assert_eq!(&bytes[..offset], b"hello world!" as &[u8]);
}

// #[test]
// fn test_bytes() {
//     let bytes = [0xde, 0xad, 0xbe, 0xef];
//     let (read, len): (&[u8], usize) = TryRead::try_read(&bytes, 4).unwrap();
//     assert_eq!(read, &[0xde, 0xad, 0xbe, 0xef]);
//     assert_eq!(len, 4);

//     assert!(bytes.pread::<&[u8]>(5).is_err());

//     let mut write = [0; 5];
//     let mut offset = 0;
//     write.gwrite(&mut offset, read).unwrap();
//     assert_eq!(write, [0xde, 0xad, 0xbe, 0xef, 0x00]);
//     assert_eq!(offset, 4);

//     assert!([0u8; 3].pwrite(0, read).is_err());
// }

#[test]
fn test_bytes() {
    let bytes: &[u8] = &[0xde, 0xad, 0xbe, 0xef];
    assert_eq!(TryRead::try_read(&bytes, ByteCtx::Length(4)).unwrap(),
               (&bytes[..], 4));

    assert!(bytes.pread_with::<&[u8]>(5, ByteCtx::Length(0)).is_err());

    let mut write = [0; 5];
    assert_eq!(TryWrite::try_write(bytes, &mut write, ()).unwrap(), 4);
    assert_eq!(&write[..4], bytes);

    assert!([0u8; 3].pwrite(0, bytes).is_err());
}

#[test]
fn test_bool() {
    let bytes = [0x00, 0x01, 0x80, 0xff];
    assert_eq!(bytes.pread::<bool>(0).unwrap(), false);
    assert_eq!(bytes.pread::<bool>(1).unwrap(), true);
    assert_eq!(bytes.pread::<bool>(2).unwrap(), true);
    assert_eq!(bytes.pread::<bool>(3).unwrap(), true);

    let mut bytes = [0u8; 2];
    bytes.pwrite(0, false).unwrap();
    bytes.pwrite(1, true).unwrap();
    assert!(bytes[0] == 0);
    assert!(bytes[1] != 0);
}

#[test]
fn test_bytes_pattern() {
    let bytes: &[u8] = b"abcde\0fghijk";

    assert_eq!(TryRead::try_read(bytes, ByteCtx::Pattern(b"abc")).unwrap(),
               (&b"abc"[..], 3));

    assert_eq!(TryRead::try_read(bytes, ByteCtx::UntilPattern(b"fg")).unwrap(),
               (&b"abcde\0fg"[..], 8));

    assert_eq!(TryRead::try_read(bytes, ByteCtx::UntilPattern(b"jk")).unwrap(),
               (&b"abcde\0fghijk"[..], 12));

    assert!(bytes
                .pread_with::<&[u8]>(0, ByteCtx::Pattern(b"bcd"))
                .is_err());
    assert!(bytes
                .pread_with::<&[u8]>(0, ByteCtx::UntilPattern(b"xyz"))
                .is_err());
    assert!(bytes
                .pread_with::<&[u8]>(10, ByteCtx::UntilPattern(b"jkl"))
                .is_err());
}

macro_rules! test_num {
    ($test_name: tt, $ty: ty, $byteorder_read_fn: tt, $byteorder_write_fn: tt) => {
        quickcheck! {
            fn $test_name (num: $ty) -> () {
                let mut bytes = [0u8; 8];          
                bytes.pwrite_with(0, num, LE).unwrap();
                let result = LittleEndian::$byteorder_read_fn(&bytes);
                assert_eq!(result, num);
                
                let mut bytes = [0u8; 8];          
                LittleEndian::$byteorder_write_fn(&mut bytes, num);
                let result: $ty = bytes.pread_with(0, LE).unwrap();
                assert_eq!(result, num);

                let mut bytes = [0u8; 8];          
                bytes.pwrite_with(0, num, BE).unwrap();
                let result = BigEndian::$byteorder_read_fn(&bytes);
                assert_eq!(result, num);
                
                let mut bytes = [0u8; 8];          
                BigEndian::$byteorder_write_fn(&mut bytes, num);
                let result: $ty = bytes.pread_with(0, BE).unwrap();
                assert_eq!(result, num);
            }
        }
    }
}

test_num!(test_u16, u16, read_u16, write_u16);
test_num!(test_u32, u32, read_u32, write_u32);
test_num!(test_u64, u64, read_u64, write_u64);
test_num!(test_i16, i16, read_i16, write_i16);
test_num!(test_i32, i32, read_i32, write_i32);
test_num!(test_i64, i64, read_i64, write_i64);
test_num!(test_f32, f32, read_f32, write_f32);
test_num!(test_f64, f64, read_f64, write_f64);