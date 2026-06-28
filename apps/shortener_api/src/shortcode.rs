use rand::RngExt;

pub const CODE_LEN: usize = 7;

const BASE62_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn generate() -> Box<str> {
    let mut rng = rand::rng();

    generate_with_rng(&mut rng)
}

fn generate_with_rng(rng: &mut impl RngExt) -> Box<str> {
    let mut code = String::with_capacity(CODE_LEN);

    for _ in 0..CODE_LEN {
        let index = rng.random_range(0..BASE62_CHARS.len());
        code.push(BASE62_CHARS[index] as char);
    }

    code.into_boxed_str()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rand::SeedableRng;
    use rand::rngs::Xoshiro256PlusPlus;

    use super::{BASE62_CHARS, CODE_LEN, generate, generate_with_rng};

    #[test]
    fn generated_code_has_expected_length() {
        assert_eq!(generate().len(), CODE_LEN);
    }

    #[test]
    fn generated_code_uses_base62_alphabet() {
        let alphabet = BASE62_CHARS.iter().copied().collect::<HashSet<_>>();

        for _ in 0..1_000 {
            let code = generate();

            assert!(code.bytes().all(|byte| alphabet.contains(&byte)));
        }
    }

    #[test]
    fn seeded_generation_is_reproducible() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(42);
        let codes = (0..5)
            .map(|_| generate_with_rng(&mut rng))
            .collect::<Vec<_>>();

        let mut same_seed = Xoshiro256PlusPlus::seed_from_u64(42);
        let same_codes = (0..5)
            .map(|_| generate_with_rng(&mut same_seed))
            .collect::<Vec<_>>();

        assert_eq!(codes, same_codes);
    }

    #[test]
    fn seeded_generation_has_reasonable_spread() {
        let mut codes = HashSet::new();
        let mut chars = HashSet::new();
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(2026);

        for _ in 0..1_000 {
            let code = generate_with_rng(&mut rng);
            chars.extend(code.bytes());
            codes.insert(code);
        }

        assert!(codes.len() > 990);
        assert!(chars.len() > 50);
    }
}
