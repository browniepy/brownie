use crate::{Context, Data, Error};

use std::collections::HashMap;

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
        // Try to get the language-specific translation
        .and_then(|locale| format(translations.other.get(locale)?, id, attr, args))
        // Otherwise, fall back on main translation
        .or_else(|| format(&translations.main, id, attr, args))
        // If this message ID is not present in any translation files whatsoever
        .unwrap_or_else(|| {
            //tracing::warn!("unknown fluent message identifier `{}`", id);
            id.to_string()
        })
}

use std::path::PathBuf;

fn project_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
}

pub fn read_ftl() -> Result<Translations, Error> {
    fn read_single_ftl(path: &std::path::Path) -> Result<(String, FluentBundle), Error> {
        // Extract locale from filename
        let locale = path.file_stem().ok_or("invalid .ftl filename")?;
        let locale = locale.to_str().ok_or("invalid filename UTF-8")?;

        // Load .ftl resource
        let file_contents = std::fs::read_to_string(path)?;
        let resource = fluent::FluentResource::try_new(file_contents)
            .map_err(|(_, e)| format!("failed to parse {:?}: {:?}", path, e))?;

        // Associate .ftl resource with locale and bundle it
        let mut bundle = FluentBundle::new_concurrent(vec![locale
            .parse()
            .map_err(|e| format!("invalid locale `{}`: {}", locale, e))?]);
        bundle
            .add_resource(resource)
            .map_err(|e| format!("failed to add resource to bundle: {:?}", e))?;

        Ok((locale.to_string(), bundle))
    }

    let root = project_root();
    let translations_dir = root.join("translations");

    Ok(Translations {
        main: read_single_ftl(&translations_dir.join("en-US.ftl"))?.1,
        other: std::fs::read_dir(translations_dir)?
            .filter_map(|file| match file {
                Ok(file) => {
                    let path = file.path();
                    if path.extension().map_or(false, |ext| ext == "ftl") {
                        Some(read_single_ftl(&path))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
            .collect::<Result<_, _>>()?,
    })
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
