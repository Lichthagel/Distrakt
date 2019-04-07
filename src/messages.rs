use serenity::builder::CreateMessage;
use trakt::models::{FullMovie, Movie};

pub fn movie<'a>(movie: &Movie, m: &'a mut CreateMessage<'a>) -> &'a mut CreateMessage<'a> {
    m.embed(|e| {
        let mut e = e.title(movie.title.to_owned()).url(format!(
            "https://trakt.tv/movies/{}",
            movie.ids.slug.as_ref().unwrap().to_owned()
        ));

        if let Some(year) = movie.year {
            e = e.field("Year", year, true)
        }
        e
    })
}

pub fn full_movie<'a>(
    movie: &FullMovie,
    m: &'a mut CreateMessage<'a>,
) -> &'a mut CreateMessage<'a> {
    m.embed(|e| {
        let mut e = e
            .title(movie.title.to_owned())
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
            e = e.field("Runtime", format!("{} runtimes", runtime), true)
        }

        e
    })
}
