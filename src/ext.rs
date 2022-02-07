use std::{error::Error, str::FromStr};

use dialoguer::theme::Theme;

pub trait OptionPromptExt {
    type Output: FromStr;
    fn unwrap_or_prompt(
        self,
        prompt: &str,
        theme: &impl Theme,
    ) -> anyhow::Result<Self::Output>;
    fn unwrap_or_prompt_password(
        self,
        prompt: &str,
        theme: &impl Theme,
    ) -> anyhow::Result<Self::Output>;
    fn unwrap_or_prompt_password_with_confirmation(
        self,
        prompt: &str,
        theme: &impl Theme,
    ) -> anyhow::Result<Self::Output>;
}

impl<T> OptionPromptExt for Option<T>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    type Output = T;

    fn unwrap_or_prompt(
        self,
        prompt: &str,
        theme: &impl Theme,
    ) -> anyhow::Result<Self::Output> {
        if let Some(val) = self {
            Ok(val)
        } else {
            let term = console::Term::stdout();
            let s: String = dialoguer::Input::with_theme(theme)
                .with_prompt(prompt)
                .show_default(true)
                .interact_on(&term)?;
            let val = T::from_str(&s)?;
            Ok(val)
        }
    }

    fn unwrap_or_prompt_password(
        self,
        prompt: &str,
        theme: &impl Theme,
    ) -> anyhow::Result<Self::Output> {
        if let Some(val) = self {
            Ok(val)
        } else {
            let term = console::Term::stdout();
            let s: String = dialoguer::Password::with_theme(theme)
                .with_prompt(prompt)
                .interact_on(&term)?;
            let val = T::from_str(&s)?;
            Ok(val)
        }
    }

    fn unwrap_or_prompt_password_with_confirmation(
        self,
        prompt: &str,
        theme: &impl Theme,
    ) -> anyhow::Result<Self::Output> {
        if let Some(val) = self {
            Ok(val)
        } else {
            let term = console::Term::stdout();
            let s: String = dialoguer::Password::with_theme(theme)
                .with_prompt(prompt)
                .with_confirmation("Confirmation", "Password Mismatch!")
                .interact_on(&term)?;
            let val = T::from_str(&s)?;
            Ok(val)
        }
    }
}
