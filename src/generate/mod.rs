use clap::Command;
use std::io::Write;

mod utils;

pub fn generate(cmd: &Command, w: &mut impl Write) {
    generate_recur(&mut vec![], cmd, w)
}

fn generate_recur(parent_commands: &mut Vec<&str>, cmd: &Command, w: &mut impl Write) {
    for flag in utils::flags(cmd) {
        let shorts = flag.get_short_and_visible_aliases().unwrap_or_default();
        let longs = flag.get_long_and_visible_aliases().unwrap_or_default();
        //
        //if let Some(shorts) = flag.get_short_and_visible_aliases() {
        //    for short in shorts {
        //        template.push_str(format!(" -s {short}").as_str());
        //    }
        //}
        //
        //if let Some(longs) = flag.get_long_and_visible_aliases() {
        //    for long in longs {
        //        template.push_str(format!(" -l {}", escape_string(long, false)).as_str());
        //    }
        //}
        //
        let description = utils::escape_help(flag.get_help().unwrap_or_default());

        //buffer.push_str(template.as_str());
        //buffer.push('\n');
    }
    unimplemented!();
}
