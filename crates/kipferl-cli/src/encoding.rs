pub fn base64_encode(input: &[u8]) -> String {
    let capacity = input.len().div_ceil(3).checked_mul(4).unwrap_or(0);
    let mut output = String::with_capacity(capacity);
    for chunk in input.chunks(3) {
        let (first, second, third) = match *chunk {
            [first, second, third] => (first, second, third),
            [first, second] => (first, second, 0),
            [first] => (first, 0, 0),
            _ => continue,
        };
        output.push(digit(first >> 2));
        output.push(digit(((first & 3) << 4) | (second >> 4)));
        output.push(if chunk.len() > 1 {
            digit(((second & 15) << 2) | (third >> 6))
        } else {
            '='
        });
        output.push(if chunk.len() > 2 { digit(third) } else { '=' });
    }
    output
}

fn digit(value: u8) -> char {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    #[expect(
        clippy::indexing_slicing,
        reason = "The six-bit mask produces an index in 0..64, exactly the alphabet length"
    )]
    let byte = ALPHABET[usize::from(value & 63)];
    char::from(byte)
}
