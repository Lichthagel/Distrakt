use serenity::builder::{CreateEmbed, CreateMessage};
use trakt::models::{FullMovie, FullShow};

pub fn full_movie<'a>(
    movie: &FullMovie,
    m: &'a mut CreateMessage<'a>,
) -> &'a mut CreateMessage<'a> {
    m.embed(|e: &mut CreateEmbed| {
        let mut e = e
            .title(format!("[Movie] {}", movie.title.to_owned()))
            .url(format!(
                "https://trakt.tv/movies/{}",
                movie.ids.slug.as_ref().unwrap().to_owned()
            ))
            .description(movie.overview.to_owned())
            .field("Released", movie.released.to_owned(), true);

        if let Some(year) = movie.year {
            e = e.field("Year", year, true);
        }

        if let Some(homepage) = movie.homepage.to_owned() {
            e = e.field("Homepage", homepage, true);
        }

        if let Some(runtime) = movie.runtime.to_owned() {
            e = e.field("Runtime", format!("{} runtimes", runtime), true);
        }

        if let Some(trailer) = movie.trailer.to_owned() {
            e = e.field("Trailer", trailer, true);
        }

        e
    })
}

pub fn full_show<'a>(show: &FullShow, m: &'a mut CreateMessage<'a>) -> &'a mut CreateMessage<'a> {
    m.embed(|e: &mut CreateEmbed| {
        let mut e = e
            .title(format!("[Show] {}", show.title.to_owned()))
            .url(format!(
                "https://trakt.tv/shows/{}",
                show.ids.slug.as_ref().unwrap().to_owned()
            ))
            .field("Aired episodes", show.aired_episodes, true);

        if let Some(overview) = show.overview.to_owned() {
            e = e.description(overview);
        }

        if let Some(year) = show.year {
            e = e.field("Year", year, true);
        }

        if let Some(homepage) = show.homepage.to_owned() {
            e = e.field("Homepage", homepage, true);
        }

        if let Some(runtime) = show.runtime.to_owned() {
            e = e.field("Runtime", format!("{} runtimes", runtime), true);
        }

        e
    })
}
