use crate::ast::{Enum, Field, Input, Struct, Variant};
use crate::attr::Attrs;
use syn::{Error, GenericArgument, PathArguments, Result, Type};

impl Input<'_> {
    pub(crate) fn validate(&self) -> Result<()> {
        match self {
            Input::Struct(input) => input.validate(),
            Input::Enum(input) => input.validate(),
        }
    }
}

impl Struct<'_> {
    fn validate(&self) -> Result<()> {
        check_non_field_attrs(&self.attrs)?;
        if let Some(transparent) = self.attrs.transparent {
            if self.fields.len() != 1 {
                return Err(Error::new_spanned(
                    transparent.original,
                    "#[error(transparent)] requires exactly one field",
                ));
            }
            if let Some(source) = self.fields.iter().find_map(|f| f.attrs.source) {
                return Err(Error::new_spanned(
                    source.original,
                    "transparent error struct can't contain #[source]",
                ));
            }
        }
        if let Some(fmt) = &self.attrs.fmt {
            return Err(Error::new_spanned(
                fmt.original,
                "#[error(fmt = ...)] is only supported in enums; for a struct, handwrite your own Display impl",
            ));
        }
        check_field_attrs(&self.fields)?;
        for field in &self.fields {
            field.validate()?;
        }
        Ok(())
    }
}

impl Enum<'_> {
    fn validate(&self) -> Result<()> {
        check_non_field_attrs(&self.attrs)?;
        let has_display = self.has_display();
        for variant in &self.variants {
            variant.validate()?;
            // Only require explicit display attributes if the enum has some display capability
            // but this specific variant lacks any display mechanism
            if has_display
                && variant.attrs.display.is_none()
                && variant.attrs.transparent.is_none()
                && variant.attrs.fmt.is_none()
                && variant.attrs.debug.is_none()
            {
                // Deny if the enum lacks #[error(debug)] fallback
                if !self.attrs.debug.is_some() {
                    return Err(Error::new_spanned(
                        variant.original,
                        "missing #[error(\"...\")] display attribute",
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Variant<'_> {
    fn validate(&self) -> Result<()> {
        check_non_field_attrs(&self.attrs)?;
        if self.attrs.transparent.is_some() {
            if self.fields.len() != 1 {
                return Err(Error::new_spanned(
                    self.original,
                    "#[error(transparent)] requires exactly one field",
                ));
            }
            if let Some(source) = self.fields.iter().find_map(|f| f.attrs.source) {
                return Err(Error::new_spanned(
                    source.original,
                    "transparent variant can't contain #[source]",
                ));
            }
        }
        check_field_attrs(&self.fields)?;
        for field in &self.fields {
            field.validate()?;
        }
        Ok(())
    }
}

impl Field<'_> {
    fn validate(&self) -> Result<()> {
        if let Some(unexpected_display_attr) = if let Some(display) = &self.attrs.display {
            Some(display.original)
        } else if let Some(fmt) = &self.attrs.fmt {
            Some(fmt.original)
        } else if let Some(debug) = &self.attrs.debug {
            Some(debug.original)
        } else {
            None
        } {
            return Err(Error::new_spanned(
                unexpected_display_attr,
                "not expected here; the #[error(...)] attribute belongs on top of a struct or an enum variant",
            ));
        }
        Ok(())
    }
}

fn check_non_field_attrs(attrs: &Attrs) -> Result<()> {
    if let Some(from) = &attrs.from {
        return Err(Error::new_spanned(
            from.original,
            "not expected here; the #[from] attribute belongs on a specific field",
        ));
    }
    if let Some(source) = &attrs.source {
        return Err(Error::new_spanned(
            source.original,
            "not expected here; the #[source] attribute belongs on a specific field",
        ));
    }
    if let Some(backtrace) = &attrs.backtrace {
        return Err(Error::new_spanned(
            backtrace,
            "not expected here; the #[backtrace] attribute belongs on a specific field",
        ));
    }
    if attrs.transparent.is_some() {
        if let Some(display) = &attrs.display {
            return Err(Error::new_spanned(
                display.original,
                "cannot have both #[error(transparent)] and a display attribute",
            ));
        }
        if let Some(fmt) = &attrs.fmt {
            return Err(Error::new_spanned(
                fmt.original,
                "cannot have both #[error(transparent)] and #[error(fmt = ...)]",
            ));
        }
        if let Some(debug) = &attrs.debug {
            return Err(Error::new_spanned(
                debug.original,
                "cannot have both #[error(transparent)] and #[error(debug)]",
            ));
        }
    } else if let (Some(display), Some(_)) = (&attrs.display, &attrs.fmt) {
        return Err(Error::new_spanned(
            display.original,
            "cannot have both #[error(fmt = ...)] and a format arguments attribute",
        ));
    } else if let (Some(display), Some(_)) = (&attrs.display, &attrs.debug) {
        return Err(Error::new_spanned(
            display.original,
            "cannot have both #[error(debug)] and a display attribute",
        ));
    } else if let (Some(fmt), Some(_)) = (&attrs.fmt, &attrs.debug) {
        return Err(Error::new_spanned(
            fmt.original,
            "cannot have both #[error(fmt = ...)] and #[error(debug)]",
        ));
    }

    Ok(())
}

fn check_field_attrs(fields: &[Field]) -> Result<()> {
    let mut from_field = None;
    let mut source_field = None;
    let mut location_field: Option<&Field> = None;
    let mut backtrace_field = None;
    let mut has_backtrace = false;
    let mut has_location = false;
    for field in fields {
        if let Some(from) = field.attrs.from {
            if from_field.is_some() {
                return Err(Error::new_spanned(
                    from.original,
                    "duplicate #[from] attribute",
                ));
            }
            from_field = Some(field);
        }
        if let Some(source) = field.attrs.source {
            if source_field.is_some() {
                return Err(Error::new_spanned(
                    source.original,
                    "duplicate #[source] attribute",
                ));
            }
            source_field = Some(field);
        }
        if let Some(backtrace) = field.attrs.backtrace {
            if backtrace_field.is_some() {
                return Err(Error::new_spanned(
                    backtrace,
                    "duplicate #[backtrace] attribute",
                ));
            }
            backtrace_field = Some(field);
            has_backtrace = true;
        }
        if let Some(location) = field.attrs.location {
            if location_field.is_some() {
                return Err(Error::new_spanned(
                    location,
                    "duplicate #[location] attribute",
                ));
            }

            location_field = Some(field);
            has_location = true;
        }
        if let Some(transparent) = field.attrs.transparent {
            return Err(Error::new_spanned(
                transparent.original,
                "#[error(transparent)] needs to go outside the enum or struct, not on an individual field",
            ));
        }
        has_backtrace |= field.is_backtrace();
        has_location |= field.is_location();
    }
    if let (Some(from_field), Some(source_field)) = (from_field, source_field) {
        if from_field.member != source_field.member {
            return Err(Error::new_spanned(
                from_field.attrs.from.unwrap().original,
                "#[from] is only supported on the source field, not any other field",
            ));
        }
    }
    if let Some(from_field) = from_field {
        let extra_fields = has_backtrace as usize + has_location as usize;
        let max_expected_fields = match (backtrace_field, location_field) {
            (Some(backtrace_field), Some(_)) => {
                2 + (from_field.member != backtrace_field.member) as usize
            }
            (Some(backtrace_field), None) => {
                1 + (from_field.member != backtrace_field.member) as usize
            }
            (None, Some(_)) => 1 + extra_fields,
            (None, None) => 1 + extra_fields,
        };
        if fields.len() > max_expected_fields {
            return Err(Error::new_spanned(
                from_field.attrs.from.unwrap().original,
                "deriving From requires no fields other than source, backtrace, and location",
            ));
        }
    }
    if let Some(source_field) = source_field.or(from_field) {
        if contains_non_static_lifetime(source_field.ty) {
            return Err(Error::new_spanned(
                &source_field.original.ty,
                "non-static lifetimes are not allowed in the source of an error, because std::error::Error requires the source is dyn Error + 'static",
            ));
        }
    }
    Ok(())
}

fn contains_non_static_lifetime(ty: &Type) -> bool {
    match ty {
        Type::Path(ty) => {
            let bracketed = match &ty.path.segments.last().unwrap().arguments {
                PathArguments::AngleBracketed(bracketed) => bracketed,
                _ => return false,
            };
            for arg in &bracketed.args {
                match arg {
                    GenericArgument::Type(ty) if contains_non_static_lifetime(ty) => return true,
                    GenericArgument::Lifetime(lifetime) if lifetime.ident != "static" => {
                        return true
                    }
                    _ => {}
                }
            }
            false
        }
        Type::Reference(ty) => ty
            .lifetime
            .as_ref()
            .map_or(false, |lifetime| lifetime.ident != "static"),
        _ => false, // maybe implement later if there are common other cases
    }
}
