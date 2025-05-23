//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{file_check, Script, ShellCore, Feeder};

pub fn source(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        eprintln!("sush: source: filename argument required");
        eprintln!("source: usage: source filename [arguments]");
        return 2;
    }

    if file_check::is_dir(&args[1]) {
        eprintln!("sush: source: {}: is a directory", &args[1]);
        return 1;
    }

    core.source_function_level += 1;
    core.source_files.push(args[1].to_string());

    core.db.position_parameters.push(args[1..].to_vec());

    let mut source = core.db.get_array_all("BASH_SOURCE");
    source.insert(0, args[1].clone());
    let _ = core.db.set_array("BASH_SOURCE", source.clone(), None);

    let mut feeder = Feeder::new("");
    feeder.set_file(&args[1]);
    feeder.main_feeder = true;
    loop {
        match feeder.feed_line(core) {
            Ok(()) => {}, 
            _ => break,
        }

        if core.return_flag {
            feeder.consume(feeder.len());
        }

        match Script::parse(&mut feeder, core, false){
            Ok(Some(mut s)) => {let _ = s.exec(core); },
            Err(e) => e.print(core),
            _ => { },
        }
    }

    source.remove(0);
    let _ = core.db.set_array("BASH_SOURCE", source, None);
    core.db.position_parameters.pop();
    core.source_function_level -= 1;
    core.source_files.pop();
    core.return_flag = false;
    core.db.exit_status
}
