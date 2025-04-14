use proc_macro::TokenStream;

use syn::parse_macro_input;
use quote::quote;

#[proc_macro]
pub fn metric(input: TokenStream) -> TokenStream {
    let parsed: SuffixParser = parse_macro_input!(input);
    quote::quote!(#parsed).into()
}

#[proc_macro]
pub fn bin(input: TokenStream) -> TokenStream {
    let mut parsed: SuffixParser = parse_macro_input!(input);
    parsed.set_style(SuffixStyle::Binary);
    quote::quote!(#parsed).into()
}

#[proc_macro]
pub fn deci(input: TokenStream) -> TokenStream {
    let mut parsed: SuffixParser = parse_macro_input!(input);
    parsed.set_style(SuffixStyle::Decimal);
    quote::quote!(#parsed).into()
}


struct SuffixParser {
    int: syn::LitInt,
    quantum: Option<Quantum>,
}

impl SuffixParser {
    /// Change the style of the suffix to `style`
    fn set_style(&mut self, style: SuffixStyle) {
        let Some(ref mut quant) = self.quantum else {
            return
        };

        quant.style = style;
    }
}

impl syn::parse::Parse for SuffixParser {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let int: syn::LitInt = input.parse()?;
        if let Some((quantum, parsed_int)) = Quantum::new(int.clone()) {
            Ok(Self {
                int: parsed_int,
                quantum: Some(quantum),
            })
        } else {
            Ok(Self {
                int,
                quantum: None,
            })
        }
    }
}

impl quote::ToTokens for SuffixParser {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let num = self.int.base10_parse::<u128>().unwrap(); // we use the biggest int type
        let multiplier = if let Some(ref quantum) = self.quantum {
            quantum.multiplier()
        } else {
            1
        };

        let num = syn::LitInt::new(&format!("{}{}",num * multiplier,self.int.suffix()), self.int.span());

        tokens.extend(quote!{#num})
    }
}


#[derive(Copy, Clone, Debug)]
struct Quantum {
    suffix: Suffix,
    style: SuffixStyle,
}

impl Quantum {

    /// Parses `int` to determine if it contains a compatible suffix.
    /// On completion returns the quantum and a parsed [syn::LitInt] without the metric suffix.
    fn new(int: syn::LitInt) -> Option<(Self, syn::LitInt)> {

        let mut ch = int.suffix().chars().peekable();

        let prefix_char = ch.next()?;
        let prefix = Suffix::from_char(prefix_char)?;

        let style = if let Some('i') = ch.peek() { // We cant pop the next char if its not 'i'
            let _ = ch.next(); // pop 'i'
            SuffixStyle::Binary
        } else {
            SuffixStyle::Decimal
        };

        let int_string = format!("{}{}",int.base10_digits(), ch.collect::<String>());

        Some((Self { suffix: prefix, style },syn::LitInt::new(&int_string,int.span())))
    }

    fn multiplier(&self) -> u128 {
        (self.style as u128).pow(self.suffix as u32)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u32)]
enum Suffix {
    Kilo = 1,
    Mega,
    Giga,
    Tera,
    Peta,
    Exa,
    Zetta,
    Yotta,
    Ronna,
    Quetta,
}

impl Suffix {
    fn from_char(c: char) -> Option<Suffix> {

        match c {
            'K' => Some(Suffix::Kilo),
            'M' => Some(Suffix::Mega),
            'G' => Some(Suffix::Giga),
            'T' => Some(Suffix::Tera),
            'P' => Some(Suffix::Peta),
            'E' => Some(Suffix::Exa),
            'Z' => Some(Suffix::Zetta),
            'Y' => Some(Suffix::Yotta),
            'R' => Some(Suffix::Ronna),
            'Q' => Some(Suffix::Quetta),
            _ => None,
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
enum SuffixStyle {
    Binary = 1028,
    Decimal = 1000,
}

#[cfg(test)]
mod test {
    use quote::quote;
    use crate::Quantum;

    #[test]
    fn quantum_multiplier_correct() {
        assert_eq!(Quantum{suffix: super::Suffix::Kilo, style: super::SuffixStyle::Decimal}.multiplier(),1000);
        assert_eq!(Quantum{suffix: super::Suffix::Exa, style: super::SuffixStyle::Decimal}.multiplier(),1000000000000000000);
    }

    #[test]
    fn parse_ok() {
        let parsed: super::SuffixParser = syn::parse2(quote! {1Ki}).unwrap();

        assert_eq!(parsed.int.base10_parse::<u128>().unwrap(),1);
        assert_eq!(parsed.quantum.as_ref().unwrap().multiplier(),1028);

    }

    #[test]
    fn largest_possible_suffix() {
        let parsed: super::SuffixParser = syn::parse2(quote! {1Qi}).unwrap();
        let _ = quote!{#parsed};
    }

    #[test]
    fn correct_int_value() {
        let parsed: super::SuffixParser = syn::parse2(quote! {1Q}).unwrap();
        let int: syn::LitInt = syn::parse2(quote!{#parsed}).unwrap();
        assert_eq!(int.base10_parse::<u128>().unwrap(),1000000000000000000000000000000);

    }

    #[test]
    fn try_build() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/fail/*.rs");
        t.pass("tests/pass/*.rs");
    }
}