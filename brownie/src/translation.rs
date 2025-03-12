use crate::{Context, Data, Error};

use std::{collections::HashMap, fs, path::Path};

type FluentBundle = fluent::bundle::FluentBundle<
    fluent::FluentResource,
    intl_memoizer::concurrent::IntlLangMemoizer,
>;

pub struct Translations {
    pub main: FluentBundle,
    pub other: HashMap<String, FluentBundle>,
}

macro_rules! translate {
    ( $ctx:ident, $id:expr $(, $argname:ident: $argvalue:expr )* $(,)? ) => {{
        #[allow(unused_mut)]
        let mut args = fluent::FluentArgs::new();
        $( args.set(stringify!($argname), $argvalue); )*

        $crate::translation::get($ctx, $id, None, Some(&args))
    }};
}

pub(crate) use translate;

pub fn format(
    bundle: &FluentBundle,
    id: &str,
    attr: Option<&str>,
    args: Option<&fluent::FluentArgs<'_>>,
) -> Option<String> {
    let message = bundle.get_message(id)?;
    let pattern = match attr {
        Some(attribute) => message.get_attribute(attribute)?.value(),
        None => message.value()?,
    };
    let formatted = bundle.format_pattern(pattern, args, &mut vec![]);
    Some(formatted.into_owned())
}

pub fn get(
    ctx: Context,
    id: &str,
    attr: Option<&str>,
    args: Option<&fluent::FluentArgs<'_>>,
) -> String {
    let translations = &ctx.data().translations;
    ctx.locale()
        .and_then(|locale| format(translations.other.get(locale)?, id, attr, args))
        .or_else(|| format(&translations.main, id, attr, args))
        .unwrap_or_else(|| format!("missing translation {}", id))
}

pub fn read_ftl() -> Result<Translations, Error> {
    fn create_bundle_for_locale(
        locale: &str,
        resources: Vec<fluent::FluentResource>,
    ) -> Result<FluentBundle, Error> {
        let mut bundle = FluentBundle::new_concurrent(vec![locale
            .parse()
            .map_err(|e| format!("invalid locale `{}`: {}", locale, e))?]);

        for resource in resources {
            bundle
                .add_resource(resource)
                .map_err(|e| format!("failed to add resource to bundle: {:?}", e))?;
        }

        Ok(bundle)
    }

    fn read_ftl_file(path: &Path) -> Result<fluent::FluentResource, Error> {
        let file_contents = fs::read_to_string(path)?;
        Ok(fluent::FluentResource::try_new(file_contents)
            .map_err(|(_, e)| format!("failed to parse {:?}: {:?}", path, e))?)
    }

    fn read_locale_directory(dir_path: &Path) -> Result<(String, FluentBundle), Error> {
        let locale = dir_path
            .file_name()
            .ok_or("invalid directory name")?
            .to_str()
            .ok_or("invalid directory name UTF-8")?
            .to_string();

        let mut resources = Vec::new();

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "ftl") {
                resources.push(read_ftl_file(&path)?);
            }
        }

        if resources.is_empty() {
            return Err(format!("no .ftl files found in directory {:?}", dir_path).into());
        }

        Ok((
            locale.clone(),
            create_bundle_for_locale(&locale, resources)?,
        ))
    }

    let main = read_locale_directory("translations/en-US".as_ref())?.1;

    let other = std::fs::read_dir("translations")?
        .map(|dir| read_locale_directory(&dir?.path()))
        .collect::<Result<_, _>>()?;

    Ok(Translations { main, other })
}

/// Given a set of language files, fills in command strings and their localizations accordingly
pub fn apply_translations(
    translations: &Translations,
    commands: &mut [poise::Command<Data, Error>],
) {
    for command in &mut *commands {
        // Add localizations
        for (locale, bundle) in &translations.other {
            // Insert localized command name and description
            let localized_command_name = match format(bundle, &command.name, None, None) {
                Some(x) => x,
                None => continue, // no localization entry => skip localization
            };
            command
                .name_localizations
                .insert(locale.clone(), localized_command_name);
            command.description_localizations.insert(
                locale.clone(),
                format(bundle, &command.name, Some("description"), None).unwrap(),
            );

            for parameter in &mut command.parameters {
                // Insert localized parameter name and description
                parameter.name_localizations.insert(
                    locale.clone(),
                    format(bundle, &command.name, Some(&parameter.name), None).unwrap(),
                );
                parameter.description_localizations.insert(
                    locale.clone(),
                    format(
                        bundle,
                        &command.name,
                        Some(&format!("{}-description", parameter.name)),
                        None,
                    )
                    .unwrap(),
                );

                // If this is a choice parameter, insert its localized variants
                for choice in &mut parameter.choices {
                    choice.localizations.insert(
                        locale.clone(),
                        format(bundle, &choice.name, None, None).unwrap(),
                    );
                }
            }
        }

        // At this point, all translation files have been applied. However, if a user uses a locale
        // we haven't explicitly inserted, there would be no translations at all -> blank texts. So,
        // we use the "main" translation file (en-US) as the non-localized strings.

        // Set fallback command name and description to en-US
        let bundle = &translations.main;
        match format(bundle, &command.name, None, None) {
            Some(x) => command.name = x,
            None => continue, // no localization entry => keep hardcoded names
        }
        command.description =
            Some(format(bundle, &command.name, Some("description"), None).unwrap());

        for parameter in &mut command.parameters {
            // Set fallback parameter name and description to en-US
            parameter.name = format(bundle, &command.name, Some(&parameter.name), None).unwrap();

            println!("{}", command.name);
            println!("{}", parameter.name);
            parameter.description = Some(
                format(
                    bundle,
                    &command.name,
                    Some(&format!("{}-description", parameter.name)),
                    None,
                )
                .unwrap(),
            );

            // If this is a choice parameter, set the choice names to en-US
            for choice in &mut parameter.choices {
                choice.name = format(bundle, &choice.name, None, None).unwrap();
            }
        }
    }
}
