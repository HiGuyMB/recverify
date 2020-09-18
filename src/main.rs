use std::env::{args, current_dir};
use std::io::{stdin, Error, Write};
use std::fs::{copy, create_dir, File, remove_file, remove_dir};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::ffi::OsString;
use std::fmt::Display;
use librec::recording::Recording;
use librec::bit_stream::BitStream;

fn dir_parents(dir: &Path) -> Vec<&Path> {
    match dir.parent() {
        Some(parent) => {
            let mut grandparents = dir_parents(parent);
            grandparents.insert(0, parent);
            grandparents
        },
        None => vec![]
    }
}

fn find_mb_exe() -> Option<PathBuf> {
    // Possible search directories for marbleblast.exe
    let base_dirs = vec![
        current_dir().ok(),
        args().collect::<Vec<_>>().first().map(|s| PathBuf::from(s)),
        args().collect::<Vec<_>>().get(1).map(|s| PathBuf::from(s)),
    ].into_iter().flat_map(|a| a).collect::<Vec<_>>();
    let test_dirs = base_dirs.iter()
        .flat_map(|pb| dir_parents(pb.as_path()))
        .collect::<Vec<_>>();

    dbg_print(format!("Dirs: {:?}", &test_dirs));

    for dir in test_dirs {
        let mut extended = PathBuf::from(dir);
        extended.push("marbleblast.exe");
        dbg_print(format!("Test: {:?}", extended));
        if extended.exists() {
            return Some(extended);
        }
    }

    return None;
}

fn wait_input() -> Result<(), Error> {
    let mut s = String::new();
    stdin().read_line(&mut s).map(|_| ())
}

fn path_name(path: &String) -> Option<String> {
    PathBuf::from(path)
        .file_name()
        .map(|oss| OsString::from(oss))
        .and_then(|oss| oss.to_str().map(|s| String::from(s)))
}

#[cfg(debug_assertions)]
fn dbg_print<S: Display>(message: S) {
    println!("{}", message);
}

#[cfg(not(debug_assertions))]
fn dbg_print<S: Display>(_: S) {
}

fn terminate_with_error<S: Display>(error: S) -> ! {
    println!("{}", error);
    println!("Press ENTER to close\n");

    wait_input().unwrap_or(());
    exit(-1);
}

fn format_time(mut t: i32) -> String {
    let mut ret = "".to_string();
    if t.is_negative() {
        t = -t;
        ret += "-";
    }
    format!("{:02}:{:02}.{:03}", (t / 1000) / 60, (t / 1000) % 60, t % 1000)
}

fn main() -> Result<(), Error> {
    let argv = args().collect::<Vec<_>>();

    if argv.len() < 2 {
        let main_exe = argv
            .first()
            .and_then(|s| path_name(s))
            .unwrap_or("recverify.exe".to_string());

        terminate_with_error(format!("No rec specified! Drag one onto {}", main_exe));
    }

    let mb_path = match find_mb_exe() {
        Some(path) => {
            dbg_print(format!("Found marbleblast.exe: {}", path.to_str().unwrap_or("<cannot display path>")));
            path
        }
        None => {
            let main_exe = path_name(&argv[0])
                .unwrap_or("recverify.exe".to_string());
            terminate_with_error(format!("Cannot find marbleblast.exe (needs to be in the same folder as {} or your rec)", main_exe));
        }
    };

    for src_path in &argv[1..] {
        // Load rec file
        let mut bit_stream = BitStream::new(fs::read(src_path)?);
        let recording = Recording::from_stream(&mut bit_stream).unwrap_or_else(|_| terminate_with_error("Failed to load rec file"));

        // From marbleblast.exe we need to inject the rec verifier script
        let mut installed_mod = false;
        let mut mod_path = PathBuf::from(mb_path.parent().unwrap());
        mod_path.extend(&["recverify"]);
        if !mod_path.exists() {
            dbg_print(format!("Creating directory {}", mod_path.to_str().unwrap_or("<cannot display path>")));
            create_dir(&mod_path)?;
            installed_mod = true;
        }

        mod_path.extend(&["main.cs"]);
        if !mod_path.exists() {
            dbg_print(format!("Writing file {}", mod_path.to_str().unwrap_or("<cannot display path>")));
            File::create(&mod_path)?.write_all(include_bytes!("../resource/main.cs"))?;
            installed_mod = true;
        }

        // Then write the demo into demos so the game can find it
        let mut demo_path = PathBuf::from(mb_path.parent().unwrap());
        demo_path.extend(&["recverify", "demos"]);
        if !demo_path.exists() {
            dbg_print(format!("Creating directory {}", demo_path.to_str().unwrap_or("<cannot display path>")));
            create_dir(&demo_path)?;
        }

        let dest_name = path_name(src_path).unwrap_or("demo.rec".to_string()).replace(" ", "_");
        demo_path.push(dest_name);
        if !demo_path.exists() {
            dbg_print(format!("Copying demo from {} to {}", src_path, demo_path.to_str().unwrap_or("<cannot display path>")));
            copy(src_path, &demo_path)?;
        }

        // Run marbleblast now!
        println!("Testing {}...", demo_path.file_name().and_then(|n| n.to_str()).unwrap());

        let game_args = vec![
            "-mod".to_string(),
            "recverify".to_string(),
            "-verify".to_string(),
            format!("recverify/demos/{}", demo_path.file_name().and_then(|n| n.to_str()).unwrap())
        ];
        let _ = Command::new(&mb_path)
            .args(game_args)
            .current_dir(mb_path.parent().unwrap())
            .output()
            .unwrap_or_else(|_| terminate_with_error("Failed to run game"));

        // Clean up
        dbg_print(format!("Deleting {}", demo_path.to_str().unwrap_or("<cannot display path>")));
        dbg_print(format!("Deleting {}", demo_path.parent().unwrap().to_str().unwrap_or("<cannot display path>")));
        remove_file(&demo_path)?;
        remove_dir(&demo_path.parent().unwrap())?;
        if installed_mod {
            dbg_print(format!("Deleting {}", &mod_path.to_str().unwrap_or("<cannot display path>")));
            dbg_print(format!("Deleting {}", &mod_path.with_extension("cs.dso").to_str().unwrap_or("<cannot display path>")));
            dbg_print(format!("Deleting {}", mod_path.parent().unwrap().to_str().unwrap_or("<cannot display path>")));
            remove_file(&mod_path)?;
            remove_file(&mod_path.with_extension("cs.dso"))?;
            remove_dir(mod_path.parent().unwrap())?;
        }

        // Check console.log output
        let mut console_path = PathBuf::from(mb_path.parent().unwrap());
        console_path.extend(&["console.log"]);

        let console_log = fs::read_to_string(console_path)?;
        let verify_start = match console_log.find("DEMO VERIFY ") {
            Some(start) => start,
            None => {
                terminate_with_error("Verify failed or didn't run?")
            }
        };

        let verify_stats = &console_log[verify_start..];
        let lines = verify_stats.split("\r\n");
        /*
        echo("DEMO VERIFY SUCCESS");
        echo("DEMO: " @ $demoArg);
        echo("MISSION: " @ $Server::MissionFile);
        echo("LEVEL NAME: " @ MissionInfo.name);
        echo("SCORE TIME: " @ $Game::ScoreTime);
        echo("ELAPSED TIME: " @ $Game::ElapsedTime);
        echo("BONUS TIME: " @ $Game::BonusTime);
        echo("GEM COUNT: " @ PlayGui.gemCount);
        echo("MAX GEMS: " @ PlayGui.maxGems);
         */
        let stats = lines.take(9).collect::<Vec<_>>();
        if stats.len() < 9 {
            // Incomplete stats or something?
            terminate_with_error("Verify stats are broken or something");
        }

        let success = stats[0] == "DEMO VERIFY SUCCESS";
        let stat_values = stats[1..9]
            .iter()
            .map(|line| line
                .split(": ")
                .skip(1)
                .take(1)
                .collect::<Vec<_>>()
                .first()
                .map(|&d| d)
                .unwrap_or_else(|| terminate_with_error("Verify stats couldn't parse"))
            )
            .collect::<Vec<_>>();

        if success {
            println!("STATUS: SUCCESS");
        } else {
            println!("STATUS: FAILURE");
        }
        println!("DEMO: {}", src_path);
        println!("MISSION: {}", stat_values[1]);
        println!("LEVEL NAME: {}", stat_values[2]);
        if success {
            println!("SCORE TIME: {} ({})", stat_values[3], format_time(i32::from_str_radix(stat_values[3], 10).unwrap_or_else(|_| terminate_with_error("Verify stats parse error"))));
            println!("ELAPSED TIME: {} ({})", stat_values[4], format_time(i32::from_str_radix(stat_values[4], 10).unwrap_or_else(|_| terminate_with_error("Verify stats parse error"))));
            println!("BONUS TIME: {} ({})", stat_values[5], format_time(i32::from_str_radix(stat_values[5], 10).unwrap_or_else(|_| terminate_with_error("Verify stats parse error"))));
            println!("GEM COUNT: {} / {}", stat_values[6], stat_values[7]);
        } else {
            println!("SCORE TIME: N/A");
            println!("ELAPSED TIME: N/A");
            println!("BONUS TIME: N/A");
            println!("GEM COUNT: N/A");
        }
        println!("FRAMES: {}", recording.frames.len());

        // Attempt to approximate FPS by ignoring the first long frames
        let mut total_frames = 0;
        let mut total_frame_time = 0f32;
        let mut is_loading = true;
        for frame in &recording.frames[10..] {
            if is_loading && frame.delta < 50 {
                is_loading = false;
            }

            if !is_loading {
                total_frames += 1;
                total_frame_time += (frame.delta as f32) / 1000f32;
            }
        }

        println!("APPROXIMATE FPS: {}", total_frames as f32 / total_frame_time);
        println!("-----------------------");
    }

    println!("Press ENTER to close\n");

    wait_input()?;
    Ok(())
}
