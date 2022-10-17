use fern::colors::{Color, ColoredLevelConfig};

pub fn setup_logger(library_name: &'static str) -> Result<(), fern::InitError> {
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::BrightBlack);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}{date}[{target}][{level}] {message}",
                color_line =
                    format_args!("\x1B[{}m", colors_line.get_color(&record.level()).to_fg_str()),
                date = chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S:%.f]"),
                target = record.target(),
                level = record.level(),
                message = message
            ))
        })
        .level(log::LevelFilter::Error)
        .level_for(library_name, log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
