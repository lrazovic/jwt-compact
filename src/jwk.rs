//! Basic support of JSON Web Keys (JWK).

use serde::{ser::SerializeMap, Serialize, Serializer};
use sha2::digest::{Digest, Output};

use core::fmt;

use crate::alloc::{Cow, ToString, Vec};

/// Conversion to [`JsonWebKey`]. This trait is implemented for all verifying keys in the crate.
pub trait ToJsonWebKey {
    /// Converts this key to a JWK presentation.
    fn to_jwk(&self) -> JsonWebKey<'_>;
}

/// Basic [JWK] functionality: serialization and creating thumbprints.
///
/// The internal format of the key is not exposed, but its fields can be indirectly accessed via
/// [`Self::thumbprint()`] method and [`Display`](fmt::Display) implementation. The latter returns
/// the presentation of the key used for hashing.
///
/// [JWK]: https://tools.ietf.org/html/rfc7517.html
pub struct JsonWebKey<'a> {
    fields: Vec<(&'static str, JwkField<'a>)>,
}

impl fmt::Debug for JsonWebKey<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_map()
            .entries(self.fields.iter().map(|(name, field)| (*name, field)))
            .finish()
    }
}

impl fmt::Display for JsonWebKey<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("{")?;
        let field_len = self.fields.len();
        for (i, (name, value)) in self.fields.iter().enumerate() {
            write!(
                formatter,
                "\"{name}\":\"{value}\"",
                name = name,
                value = value
            )?;
            if i + 1 < field_len {
                formatter.write_str(",")?;
            }
        }
        formatter.write_str("}")
    }
}

impl<'a> JsonWebKey<'a> {
    /// Instantiates a key builder.
    pub fn builder(key_type: &'static str) -> JsonWebKeyBuilder<'a> {
        let mut fields = Vec::with_capacity(4);
        fields.push(("kty", JwkField::Str(key_type)));
        JsonWebKeyBuilder {
            inner: Self { fields },
        }
    }

    /// Computes a thumbprint of this JWK as per [RFC 7638].
    ///
    /// [RFC 7638]: https://tools.ietf.org/html/rfc7638
    pub fn thumbprint<D: Digest>(&self) -> Output<D> {
        D::digest(self.to_string().as_bytes())
    }
}

impl Serialize for JsonWebKey<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.fields.len()))?;
        for (name, value) in &self.fields {
            map.serialize_entry(*name, &value.to_string())?;
        }
        map.end()
    }
}

/// Builder for [`JsonWebKey`].
#[derive(Debug)]
pub struct JsonWebKeyBuilder<'a> {
    inner: JsonWebKey<'a>,
}

impl<'a> JsonWebKeyBuilder<'a> {
    /// Adds a string field with the specified name.
    ///
    /// # Panics
    ///
    /// Panics if the field with this name is already present.
    pub fn with_str_field(mut self, field_name: &'static str, value: &'static str) -> Self {
        let existing_field = self
            .inner
            .fields
            .iter()
            .find(|(name, _)| *name == field_name);
        if let Some((_, old_value)) = existing_field {
            panic!("Field `{}` is already defined: {:?}", field_name, old_value);
        }
        self.inner.fields.push((field_name, JwkField::Str(value)));
        self
    }

    /// Adds a byte field with the specified name. Bytes can be borrowed from the key, or
    /// can be instantiated for this method.
    ///
    /// # Panics
    ///
    /// Panics if the field with this name is already present.
    pub fn with_bytes_field(
        mut self,
        field_name: &'static str,
        value: impl Into<Cow<'a, [u8]>>,
    ) -> Self {
        let existing_field = self
            .inner
            .fields
            .iter()
            .find(|(name, _)| *name == field_name);
        if let Some((_, old_value)) = existing_field {
            panic!("Field `{}` is already defined: {:?}", field_name, old_value);
        }
        self.inner
            .fields
            .push((field_name, JwkField::Bytes(value.into())));
        self
    }

    /// Consumes this builder creating [`JsonWebKey`].
    pub fn build(self) -> JsonWebKey<'a> {
        let mut inner = self.inner;
        inner.fields.sort_unstable_by_key(|(name, _)| *name);
        inner
    }
}

#[derive(Debug)]
enum JwkField<'a> {
    Str(&'static str),
    Bytes(Cow<'a, [u8]>),
}

impl fmt::Display for JwkField<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Str(s) => formatter.write_str(s),
            Self::Bytes(bytes) => {
                let encoded = base64::encode_config(bytes, base64::URL_SAFE_NO_PAD);
                formatter.write_str(&encoded)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializing_jwk() {
        let jwk = JsonWebKey {
            fields: vec![
                ("crv", JwkField::Str("Ed25519")),
                ("kty", JwkField::Str("OKP")),
                ("x", JwkField::Bytes(Cow::Borrowed(b"test"))),
            ],
        };
        let json = serde_json::to_value(&jwk).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "crv": "Ed25519", "kty": "OKP", "x": "dGVzdA" })
        );
    }
}
