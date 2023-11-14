//! This crate provides a derive macro for the `BFieldCodec` trait.

extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Attribute;
use syn::DeriveInput;
use syn::Field;
use syn::Fields;
use syn::Ident;
use syn::Type;
use syn::Variant;

/// Derives `BFieldCodec` for structs.
///
/// Fields that should not be serialized can be ignored by annotating them with
/// `#[bfield_codec(ignore)]`.
/// Ignored fields must implement [`Default`].
///
/// ### Example
///
/// ```ignore
/// #[derive(BFieldCodec)]
/// struct Foo {
///    bar: u64,
///    #[bfield_codec(ignore)]
///    ignored: usize,
/// }
/// let foo = Foo { bar: 42, ignored: 7 };
/// let encoded = foo.encode();
/// let decoded = Foo::decode(&encoded).unwrap();
/// assert_eq!(foo.bar, decoded.bar);
/// ```
///
/// ### Known limitations
/// ```
#[proc_macro_derive(BFieldCodec, attributes(bfield_codec))]
pub fn bfieldcodec_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    BFieldCodecDeriveBuilder::new(ast).build().into()
}

enum BFieldCodecDeriveType {
    StructWithNamedFields,
    StructWithUnnamedFields,
    Enum,
}

struct BFieldCodecDeriveBuilder {
    name: syn::Ident,
    derive_type: BFieldCodecDeriveType,
    generics: syn::Generics,
    attributes: Vec<Attribute>,

    named_included_fields: Vec<Field>,
    named_ignored_fields: Vec<Field>,

    unnamed_fields: Vec<Field>,

    variants: Option<Punctuated<Variant, syn::token::Comma>>,

    encode_statements: Vec<TokenStream>,
    decode_function_body: TokenStream,
    static_length_body: TokenStream,
}

impl BFieldCodecDeriveBuilder {
    fn new(ast: DeriveInput) -> Self {
        let derive_type = Self::extract_derive_type(&ast);

        let named_fields = Self::extract_named_fields(&ast);
        let (ignored_fields, included_fields) = named_fields
            .iter()
            .cloned()
            .partition::<Vec<_>, _>(Self::field_is_ignored);

        let unnamed_fields = Self::extract_unnamed_fields(&ast);
        let variants = Self::extract_variants(&ast);

        let name = ast.ident;

        Self {
            name,
            derive_type,
            generics: ast.generics,
            attributes: ast.attrs,

            named_included_fields: included_fields,
            named_ignored_fields: ignored_fields,
            unnamed_fields,
            variants,

            encode_statements: vec![],
            decode_function_body: quote! {},
            static_length_body: quote! {},
        }
    }

    fn extract_derive_type(ast: &DeriveInput) -> BFieldCodecDeriveType {
        match &ast.data {
            syn::Data::Struct(syn::DataStruct {
                fields: Fields::Named(_),
                ..
            }) => BFieldCodecDeriveType::StructWithNamedFields,
            syn::Data::Struct(syn::DataStruct {
                fields: Fields::Unnamed(_),
                ..
            }) => BFieldCodecDeriveType::StructWithUnnamedFields,
            syn::Data::Enum(_) => BFieldCodecDeriveType::Enum,
            _ => panic!("expected a struct with named fields, with unnamed fields, or an enum"),
        }
    }

    fn extract_named_fields(ast: &DeriveInput) -> Vec<Field> {
        match &ast.data {
            syn::Data::Struct(syn::DataStruct {
                fields: Fields::Named(fields),
                ..
            }) => fields.named.iter().cloned().collect::<Vec<_>>(),
            _ => vec![],
        }
    }

    fn extract_unnamed_fields(ast: &DeriveInput) -> Vec<Field> {
        match &ast.data {
            syn::Data::Struct(syn::DataStruct {
                fields: Fields::Unnamed(fields),
                ..
            }) => fields.unnamed.iter().cloned().collect::<Vec<_>>(),
            _ => vec![],
        }
    }

    fn extract_variants(ast: &DeriveInput) -> Option<Punctuated<Variant, Comma>> {
        match &ast.data {
            syn::Data::Enum(data_enum) => Some(data_enum.variants.clone()),
            _ => None,
        }
    }

    fn field_is_ignored(field: &Field) -> bool {
        let mut relevant_attributes = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("bfield_codec"));
        let attribute = match relevant_attributes.clone().count() {
            0 => return false,
            1 => relevant_attributes.next().unwrap(),
            _ => panic!("field must have at most 1 `bfield_codec` attribute"),
        };
        attribute
            .parse_nested_meta(|meta| match meta.path.get_ident() {
                Some(ident) if ident == "ignore" => Ok(()),
                Some(ident) => Err(meta.error(format!("Unknown identifier \"{ident}\"."))),
                _ => Err(meta.error("Expected an identifier.")),
            })
            .unwrap();

        // unwrap only succeeds if the attribute is `ignore`
        true
    }

    fn build(mut self) -> TokenStream {
        self.add_trait_bounds_to_generics();
        self.build_methods();
        self.into_token_stream()
    }

    fn add_trait_bounds_to_generics(&mut self) {
        let ignored_generics = self.extract_ignored_generics_list();
        let ignored_generics = self.recursively_collect_all_ignored_generics(ignored_generics);

        for param in &mut self.generics.params {
            let syn::GenericParam::Type(type_param) = param else {
                continue;
            };
            if ignored_generics.contains(&type_param.ident) {
                continue;
            }
            type_param.bounds.push(syn::parse_quote!(BFieldCodec));
        }
    }

    fn extract_ignored_generics_list(&self) -> Vec<syn::Ident> {
        self.attributes
            .iter()
            .flat_map(Self::extract_ignored_generics)
            .collect()
    }

    fn extract_ignored_generics(attr: &Attribute) -> Vec<Ident> {
        if !attr.path().is_ident("bfield_codec") {
            return vec![];
        }

        let mut ignored_generics = vec![];
        attr.parse_nested_meta(|meta| match meta.path.get_ident() {
            Some(ident) if ident == "ignore" => {
                ignored_generics.push(ident.to_owned());
                Ok(())
            }
            Some(ident) => Err(meta.error(format!("Unknown identifier \"{ident}\"."))),
            _ => Err(meta.error("Expected an identifier.")),
        })
        .unwrap();
        ignored_generics
    }

    /// For all ignored fields, add all type identifiers (including, recursively, the type
    /// identifiers of generic type arguments) to the list of ignored type identifiers.
    fn recursively_collect_all_ignored_generics(
        &self,
        mut ignored_generics: Vec<Ident>,
    ) -> Vec<Ident> {
        let mut ignored_types = self
            .named_ignored_fields
            .iter()
            .map(|ignored_field| ignored_field.ty.clone())
            .collect::<Vec<_>>();
        while !ignored_types.is_empty() {
            let ignored_type = ignored_types[0].clone();
            ignored_types = ignored_types[1..].to_vec();
            let Type::Path(type_path) = ignored_type else {
                continue;
            };
            for segment in type_path.path.segments.into_iter() {
                ignored_generics.push(segment.ident);
                let syn::PathArguments::AngleBracketed(generic_arguments) = segment.arguments
                else {
                    continue;
                };
                for generic_argument in generic_arguments.args.into_iter() {
                    let syn::GenericArgument::Type(t) = generic_argument else {
                        continue;
                    };
                    ignored_types.push(t.clone());
                }
            }
        }
        ignored_generics
    }

    fn build_methods(&mut self) {
        match self.derive_type {
            BFieldCodecDeriveType::StructWithNamedFields => {
                self.build_methods_for_struct_with_named_fields()
            }
            BFieldCodecDeriveType::StructWithUnnamedFields => {
                self.build_methods_for_struct_with_unnamed_fields()
            }
            BFieldCodecDeriveType::Enum => self.build_methods_for_enum(),
        }
    }

    fn build_methods_for_struct_with_named_fields(&mut self) {
        self.build_encode_statements_for_struct_with_named_fields();
        self.build_decode_function_body_for_struct_with_named_fields();
        let included_fields = self.named_included_fields.clone();
        self.build_static_length_body_for_struct(&included_fields);
    }

    fn build_methods_for_struct_with_unnamed_fields(&mut self) {
        self.build_encode_statements_for_struct_with_unnamed_fields();
        self.build_decode_function_body_for_struct_with_unnamed_fields();
        let included_fields = self.unnamed_fields.clone();
        self.build_static_length_body_for_struct(&included_fields);
    }

    fn build_methods_for_enum(&mut self) {
        self.build_encode_statements_for_enum();
        self.build_decode_function_body_for_enum();
        self.build_static_length_body_for_enum();
    }

    fn build_encode_statements_for_struct_with_named_fields(&mut self) {
        let included_field_names = self
            .named_included_fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap().to_owned());
        let included_field_types = self
            .named_included_fields
            .iter()
            .map(|field| field.ty.clone());
        self.encode_statements = included_field_names
            .clone()
            .zip(included_field_types.clone())
            .map(|(field_name, field_type)| {
                quote! {
                    let #field_name:
                        Vec<::twenty_first::shared_math::b_field_element::BFieldElement> =
                            self.#field_name.encode();
                    if <#field_type as ::twenty_first::shared_math::bfield_codec::BFieldCodec>
                        ::static_length().is_none() {
                        elements.push(
                            ::twenty_first::shared_math::b_field_element::BFieldElement::new(
                                #field_name.len() as u64
                            )
                        );
                    }
                    elements.extend(#field_name);
                }
            })
            .collect();
    }

    fn build_encode_statements_for_struct_with_unnamed_fields(&mut self) {
        let field_types = self.unnamed_fields.iter().map(|field| field.ty.clone());
        let indices: Vec<_> = (0..self.unnamed_fields.len())
            .map(syn::Index::from)
            .collect();
        let field_names: Vec<_> = indices
            .iter()
            .map(|i| quote::format_ident!("field_value_{}", i.index))
            .collect();
        self.encode_statements = indices
            .iter()
            .zip(field_types.clone())
            .zip(field_names.clone())
            .map(|((idx, field_type), field_name)| {
                quote! {
                    let #field_name:
                        Vec<::twenty_first::shared_math::b_field_element::BFieldElement> =
                            self.#idx.encode();
                    if <#field_type as ::twenty_first::shared_math::bfield_codec::BFieldCodec>
                        ::static_length().is_none() {
                        elements.push(
                            ::twenty_first::shared_math::b_field_element::BFieldElement::new(
                                #field_name.len() as u64
                            )
                        );
                    }
                    elements.extend(#field_name);
                }
            })
            .collect();
    }

    fn build_encode_statements_for_enum(&mut self) {
        let encode_clauses = self
            .variants
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, v)| self.generate_encode_clause_for_variant(i, &v.ident, &v.fields));
        let encode_match_statement = quote! {
            match self {
                #( #encode_clauses , )*
            }
        };
        self.encode_statements = vec![encode_match_statement];
    }

    fn generate_encode_clause_for_variant(
        &self,
        variant_index: usize,
        variant_name: &Ident,
        associated_data: &Fields,
    ) -> TokenStream {
        if associated_data.is_empty() {
            return quote! {
                Self::#variant_name => {
                    elements.push(::twenty_first::shared_math::b_field_element::BFieldElement::new(
                        #variant_index as u64)
                    );
                }
            };
        }

        let field_encoders = associated_data.iter().enumerate().map(|(field_index, ad)| {
            let field_name = self.enum_variant_field_name(variant_index, field_index);
            let field_type = ad.ty.clone();
            let field_encoding =
                quote::format_ident!("variant_{}_field_{}_encoding", variant_index, field_index);
            quote! {
                let #field_encoding:
                    Vec<::twenty_first::shared_math::b_field_element::BFieldElement> =
                        #field_name.encode();
                if <#field_type as ::twenty_first::shared_math::bfield_codec::BFieldCodec>
                    ::static_length().is_none() {
                    elements.push(
                        ::twenty_first::shared_math::b_field_element::BFieldElement::new(
                            #field_encoding.len() as u64
                        )
                    );
                }
                elements.extend(#field_encoding);
            }
        });

        let field_names = associated_data
            .iter()
            .enumerate()
            .map(|(field_index, _field)| self.enum_variant_field_name(variant_index, field_index));

        quote! {
            Self::#variant_name ( #( #field_names , )* ) => {
                elements.push(
                    ::twenty_first::shared_math::b_field_element::BFieldElement::new(
                        #variant_index as u64
                    )
                );
                #( #field_encoders )*
            }
        }
    }

    fn build_decode_function_body_for_struct_with_named_fields(&mut self) {
        let decode_statements = self
            .named_included_fields
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                self.generate_decode_statement_for_field(field_name, &field.ty)
            })
            .collect::<Vec<_>>();

        let included_field_names = self.named_included_fields.iter().map(|field| {
            let field_name = field.ident.as_ref().unwrap().to_owned();
            quote! { #field_name }
        });
        let ignored_field_names = self.named_ignored_fields.iter().map(|field| {
            let field_name = field.ident.as_ref().unwrap().to_owned();
            quote! { #field_name }
        });
        let name = self.name.to_string();

        self.decode_function_body = quote! {
            #(#decode_statements)*
            if !sequence.is_empty() {
                anyhow::bail!(
                    "Could not decode {}: sequence too long. ({} elements remaining)",
                    #name,
                    sequence.len()
                );
            }
            Ok(Box::new(Self {
                #(#included_field_names,)*
                #(#ignored_field_names: Default::default(),)*
            }))
        };
    }

    fn build_decode_function_body_for_struct_with_unnamed_fields(&mut self) {
        let field_names: Vec<_> = (0..self.unnamed_fields.len())
            .map(|i| quote::format_ident!("field_value_{}", i))
            .collect();
        let decode_statements = field_names
            .iter()
            .zip(self.unnamed_fields.iter())
            .map(|(field_name, field)| {
                self.generate_decode_statement_for_field(field_name, &field.ty)
            })
            .collect::<Vec<_>>();

        let name = self.name.to_string();

        self.decode_function_body = quote! {
            #(#decode_statements)*
            if !sequence.is_empty() {
                anyhow::bail!(
                    "Could not decode {}: sequence too long. ({} elements remaining)",
                    #name,
                    sequence.len()
                );
            }
            Ok(Box::new(Self ( #(#field_names,)* )))
        };
    }

    fn generate_decode_statement_for_field(
        &self,
        field_name: &Ident,
        field_type: &Type,
    ) -> TokenStream {
        let name = self.name.to_string();
        let field_name_as_string_literal = field_name.to_string();
        quote! {
            let (#field_name, sequence) = {
                let maybe_fields_static_length =
                    <#field_type as ::twenty_first::shared_math::bfield_codec::BFieldCodec>
                        ::static_length();
                let field_has_dynamic_length = maybe_fields_static_length.is_none();
                if sequence.is_empty() && field_has_dynamic_length {
                    anyhow::bail!(
                        "Cannot decode field {} of {}: sequence is empty.",
                        #field_name_as_string_literal,
                        #name,
                    );
                }
                let (len, sequence) = match maybe_fields_static_length {
                    Some(len) => (len, sequence),
                    None => (sequence[0].value() as usize, &sequence[1..]),
                };
                if sequence.len() < len {
                    anyhow::bail!(
                        "Cannot decode field {} of {}: sequence too short.",
                        #field_name_as_string_literal,
                        #name,
                    );
                }
                let decoded =
                    *<#field_type as ::twenty_first::shared_math::bfield_codec::BFieldCodec>
                        ::decode(
                            &sequence[..len]
                        ).map_err(|e| {
                            anyhow::anyhow!(
                                "Could not decode field {} of {}: {}",
                                #field_name_as_string_literal,
                                #name,
                                e,
                            )
                    }
                )?;
                (decoded, &sequence[len..])
            };
        }
    }

    fn build_decode_function_body_for_enum(&mut self) {
        let decode_clauses = self
            .variants
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, v)| self.generate_decode_clause_for_variant(i, &v.ident, &v.fields));
        let match_clauses = decode_clauses
            .enumerate()
            .map(|(index, decode_clause)| quote! { #index => { #decode_clause } });

        let name = self.name.to_string();
        self.decode_function_body = quote! {
            if sequence.is_empty() {
                anyhow::bail!("Cannot decode {}: sequence is empty", #name);
            }
            let (variant_index, sequence) = (sequence[0].value() as usize, &sequence[1..]);
            match variant_index {
                #(#match_clauses ,)*
                other_index => anyhow::bail!(
                    "Cannot decode variant {other_index} of {}: invalid variant index", #name
                ),
            }
        };
    }

    fn generate_decode_clause_for_variant(
        &self,
        variant_index: usize,
        variant_name: &Ident,
        associated_data: &Fields,
    ) -> TokenStream {
        let name = self.name.to_string();

        if associated_data.is_empty() {
            return quote! {
                if !sequence.is_empty() {
                    anyhow::bail!("Cannot decode {}: sequence too long.", #name);
                }
                Ok(Box::new(Self::#variant_name))
            };
        }
        let field_decoders = associated_data.iter().enumerate().map(|(field_index, field)| {
                let field_type = field.ty.clone();
                let field_name = self.enum_variant_field_name(variant_index, field_index);
                let field_value =
                    quote::format_ident!("variant_{}_field_{}_value", variant_index, field_index);
                quote! {
                let (#field_value, sequence) = {
                    let maybe_fields_static_length =
                        <#field_type as ::twenty_first::shared_math::bfield_codec::BFieldCodec>::static_length();
                    let field_has_dynamic_length = maybe_fields_static_length.is_none();
                    if sequence.is_empty() && field_has_dynamic_length {
                        anyhow::bail!(
                                "Cannot decode variant {} field {} of {}: sequence is empty.",
                                #variant_index,
                                #field_index,
                                #name,
                            );
                    }
                    let (len, sequence) = match maybe_fields_static_length {
                        Some(len) => (len, sequence),
                        None => (sequence[0].value() as usize, &sequence[1..]),
                    };
                    if sequence.len() < len {
                        anyhow::bail!(
                                "Cannot decode variant {} field {} of {}: sequence too short.",
                                #variant_index,
                                #field_index,
                                #name,
                            );
                    }
                    let decoded =
                        *<#field_type as ::twenty_first::shared_math::bfield_codec::BFieldCodec>::decode(
                            &sequence[..len]
                        )?;
                    (decoded, &sequence[len..])
                };
                let #field_name = #field_value;
            }
            }).fold(quote!{}, |l, r| quote!{#l #r});
        let field_names = associated_data
            .iter()
            .enumerate()
            .map(|(field_index, _field)| self.enum_variant_field_name(variant_index, field_index));
        quote! {
            #field_decoders
            if !sequence.is_empty() {
                anyhow::bail!("Cannot decode {}: sequence too long.", #name);
            }
            Ok(Box::new(Self::#variant_name ( #( #field_names , )* )))
        }
    }

    fn enum_variant_field_name(&self, variant_index: usize, field_index: usize) -> syn::Ident {
        quote::format_ident!("variant_{}_field_{}", variant_index, field_index)
    }

    fn build_static_length_body_for_struct(&mut self, fields: &[Field]) {
        let field_types = fields
            .iter()
            .map(|field| field.ty.clone())
            .collect::<Vec<_>>();
        let num_fields = field_types.len();
        self.static_length_body = quote! {
            let field_lengths : [Option<usize>; #num_fields] = [
                #(
                    <#field_types as
                    ::twenty_first::shared_math::bfield_codec::BFieldCodec>::static_length(),
                )*
            ];
            if field_lengths.iter().all(|fl| fl.is_some() ) {
                Some(field_lengths.iter().map(|fl| fl.unwrap()).sum())
            }
            else {
                None
            }
        };
    }

    fn build_static_length_body_for_enum(&mut self) {
        let variants = self.variants.as_ref().unwrap();
        let no_variants_have_associated_data = variants.iter().all(|v| v.fields.is_empty());
        if no_variants_have_associated_data {
            self.static_length_body = quote! {Some(1)};
            return;
        }

        let num_variants = variants.len();
        if num_variants == 0 {
            self.static_length_body = quote! {Some(0)};
            return;
        }

        // some variants have associated data
        // if all variants encode to the same length, the length is statically known anyway
        let variant_lengths = variants
            .iter()
            .map(|variant| {
                let fields = variant.fields.clone();
                let field_lengths = fields .iter().map(|f| quote!{
                        <#f as ::twenty_first::shared_math::bfield_codec::BFieldCodec>::static_length()
                    }
                );
                let num_fields = fields.len();
                quote!{{
                    let field_lengths: [Option<usize>; #num_fields] = [ #( #field_lengths , )* ];
                    if field_lengths.iter().all(|fl| fl.is_some()) {
                        Some(field_lengths.iter().map(|fl|fl.unwrap()).sum())
                    } else {
                        None
                    }
                }}
            }
        )
        .collect::<Vec<_>>();

        self.static_length_body = quote! {
                let variant_lengths : [Option<usize>; #num_variants] = [ #( #variant_lengths , )* ];
                if variant_lengths.iter().all(|fl| fl.is_some() ) &&
                    variant_lengths.iter().tuple_windows().all(|(l, r)| l.unwrap() == r.unwrap()) {
                    variant_lengths[0]
                }
                else {
                    None
                }

        };
    }

    fn into_token_stream(self) -> TokenStream {
        let name = self.name;
        let decode_function_body = self.decode_function_body;
        let encode_statements = self.encode_statements;
        let static_length_body = self.static_length_body;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            impl #impl_generics ::twenty_first::shared_math::bfield_codec::BFieldCodec
            for #name #ty_generics #where_clause {
                type Error = anyhow::Error;

                fn decode(
                    sequence: &[::twenty_first::shared_math::b_field_element::BFieldElement],
                ) -> Result<Box<Self>, Self::Error> {
                    #decode_function_body
                }

                fn encode(&self) -> Vec<::twenty_first::shared_math::b_field_element::BFieldElement> {
                    let mut elements = Vec::new();
                    #(#encode_statements)*
                    elements
                }

                fn static_length() -> Option<usize> {
                    #static_length_body
                }
            }
        }
    }
}
