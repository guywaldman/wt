use std::io::Write;

use clap::{Command, ValueEnum};
use clap_complete::Shell as CompletionShell;

use crate::Result;

/// Responsible for `wt init` shell setup code generation.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum InitShell {
    Bash,
    Fish,
    Zsh,
}

impl InitShell {
    fn completion_shell(self) -> CompletionShell {
        match self {
            Self::Bash => CompletionShell::Bash,
            Self::Fish => CompletionShell::Fish,
            Self::Zsh => CompletionShell::Zsh,
        }
    }

    fn integration(self) -> &'static str {
        match self {
            Self::Bash | Self::Zsh => POSIX_SHELL_INTEGRATION,
            Self::Fish => FISH_SHELL_INTEGRATION,
        }
    }
}

/// Emit completions plus the shell function that lets `wt switch` change cwd.
pub(crate) fn write(shell: InitShell, command: Command, mut out: impl Write) -> Result<()> {
    let mut command = command.mut_subcommand("switch", |command| command.hide(false));
    clap_complete::generate(shell.completion_shell(), &mut command, "wt", &mut out);
    writeln!(out)?;
    write!(out, "{}", shell.integration())?;
    Ok(())
}

const POSIX_SHELL_INTEGRATION: &str = r#"# Wrap `wt switch` so it can change the current shell's cwd.
wt() {
  if [ "${1:-}" = "switch" ]; then
    shift
    local target
    target="$(command wt switch "$@")" || return
    cd "$target"
  else
    command wt "$@"
  fi
}
"#;

const FISH_SHELL_INTEGRATION: &str = r#"# Wrap `wt switch` so it can change the current shell's cwd.
function wt
    if test (count $argv) -gt 0; and test "$argv[1]" = switch
        set -l target (command wt switch $argv[2..-1])
        if test $status -ne 0
            return $status
        end
        cd "$target"
    else
        command wt $argv
    end
end
"#;
