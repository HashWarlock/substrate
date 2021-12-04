// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::pallet::{parse::error::VariantField, Def};
use frame_support_procedural_tools::get_doc_literals;

///
/// * impl various trait on Error
pub fn expand_error(def: &mut Def) -> proc_macro2::TokenStream {
	let frame_support = &def.frame_support;
	let frame_system = &def.frame_system;
	let pallet_ident = &def.pallet_struct.pallet;
	let pallet_type_impl_gen = &def.type_impl_generics(def.pallet_struct.attr_span);
	let pallet_type_use_gen = &def.type_use_generics(def.pallet_struct.attr_span);
	let config_where_clause = &def.config.where_clause;

	let error = if let Some(error) = &def.error {
		error
	} else {
		return quote::quote! {
			impl<#pallet_type_impl_gen> #frame_support::traits::ErrorCompactnessTest
				for #pallet_ident<#pallet_type_use_gen> #config_where_clause {}
		}
	};

	let error_ident = &error.error;
	let type_impl_gen = &def.type_impl_generics(error.attr_span);
	let type_use_gen = &def.type_use_generics(error.attr_span);

	let phantom_variant: syn::Variant = syn::parse_quote!(
		#[doc(hidden)]
		#[codec(skip)]
		__Ignore(
			#frame_support::sp_std::marker::PhantomData<(#type_use_gen)>,
			#frame_support::Never,
		)
	);

	let as_str_matches = error.variants.iter().map(|(variant, field_ty, _)| {
		let variant_str = format!("{}", variant);
		match field_ty {
			Some(VariantField { is_named: true, .. }) => {
				quote::quote_spanned!(error.attr_span => Self::#variant { .. } => #variant_str,)
			},
			Some(VariantField { is_named: false, .. }) => {
				quote::quote_spanned!(error.attr_span => Self::#variant(..) => #variant_str,)
			},
			None => {
				quote::quote_spanned!(error.attr_span => Self::#variant => #variant_str,)
			},
		}
	});

	let error_item = {
		let item = &mut def.item.content.as_mut().expect("Checked by def parser").1[error.index];
		if let syn::Item::Enum(item) = item {
			item
		} else {
			unreachable!("Checked by error parser")
		}
	};

	error_item.variants.insert(0, phantom_variant);
	// derive TypeInfo for error metadata
	error_item.attrs.push(syn::parse_quote! {
		#[derive(
			#frame_support::codec::Encode,
			#frame_support::codec::Decode,
			#frame_support::scale_info::TypeInfo,
			#frame_support::CompactPalletError,
		)]
	});
	error_item.attrs.push(syn::parse_quote!(
		#[scale_info(skip_type_params(#type_use_gen), capture_docs = "always")]
	));

	if get_doc_literals(&error_item.attrs).is_empty() {
		error_item.attrs.push(syn::parse_quote!(
			#[doc = r"
			Custom [dispatch errors](https://docs.substrate.io/v3/runtime/events-and-errors)
			of this pallet.
			"]
		));
	}

	quote::quote_spanned!(error.attr_span =>
		impl<#pallet_type_impl_gen> #frame_support::traits::ErrorCompactnessTest
			for #pallet_ident<#pallet_type_use_gen> #config_where_clause
		{
			fn error_compactness_test() {
				assert!(
					<
						#error_ident<#type_use_gen> as #frame_support::traits::CompactPalletError
					>::check_compactness(),
					"Pallet error enum is not the most compact possible"
				);
			}
		}

		impl<#type_impl_gen> #frame_support::sp_std::fmt::Debug for #error_ident<#type_use_gen>
			#config_where_clause
		{
			fn fmt(&self, f: &mut #frame_support::sp_std::fmt::Formatter<'_>)
				-> #frame_support::sp_std::fmt::Result
			{
				f.write_str(self.as_str())
			}
		}

		impl<#type_impl_gen> #error_ident<#type_use_gen> #config_where_clause {
			pub fn as_str(&self) -> &'static str {
				match &self {
					Self::__Ignore(_, _) => unreachable!("`__Ignore` can never be constructed"),
					#( #as_str_matches )*
				}
			}
		}

		impl<#type_impl_gen> From<#error_ident<#type_use_gen>> for &'static str
			#config_where_clause
		{
			fn from(err: #error_ident<#type_use_gen>) -> &'static str {
				err.as_str()
			}
		}

		impl<#type_impl_gen> From<#error_ident<#type_use_gen>>
			for #frame_support::sp_runtime::DispatchError
			#config_where_clause
		{
			fn from(err: #error_ident<#type_use_gen>) -> Self {
				let index = <
					<T as #frame_system::Config>::PalletInfo
					as #frame_support::traits::PalletInfo
				>::index::<Pallet<#type_use_gen>>()
					.expect("Every active module has an index in the runtime; qed") as u8;
				let mut encoded = err.encode();
				encoded.resize(#frame_support::MAX_NESTED_PALLET_ERROR_DEPTH, 0);

				#frame_support::sp_runtime::DispatchError::Module {
					index,
					error: encoded.try_into().expect("encoded error is resized to be equal to 4 bytes; qed"),
					message: Some(err.as_str()),
				}
			}
		}
	)
}
