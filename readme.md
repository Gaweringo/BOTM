<p align="center">
  <a href="" rel="noopener">
 <img width=200px height=200px src="assets/BOTM Logo.jpg" alt="Cat shooting laser eyes (BOTM Logo)"></a>
</p>

<h3 align="center">BOTM</h3>
<h4 align="center">Bangers Of The Month</h4>

<div align="center">

[![Status](https://img.shields.io/github/v/release/Gaweringo/BOTM?display_name=tag&include_prereleases.svg)](https://github.com/Gaweringo/BOTM/releases)
[![GitHub Issues](https://img.shields.io/github/issues/Gaweringo/BOTM.svg)](https://github.com/Gaweringo/BOTM/issues)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](/LICENSE)

</div>

---

<p align="center"> A small program to create a playlist with your top Spotify songs for the last four weeks.
    <br>
</p>

## 📝 Table of Contents

- [About](#about)
- [Getting Started](#getting_started)
- [Usage](#usage)
- [Built Using](#built_using)
- [Authors](#authors)
- [Acknowledgments](#acknowledgement)

## 🧐 About <a name = "about"></a>

This program lets you create a playlist with your top songs of the past four weeks on Spotify. It also lets you schedule the creation of this playlist automatically using the Windows Task Scheduler.

## 🏁 Getting Started <a name = "getting_started"></a>

<!-- This section explains  -->

### Prerequisites

Create a new app on [the Spotify developer dashboard](https://developer.spotify.com/dashboard/). Give it a name like BOTM so, you can remember what the app is for.

After you created the app you should see the its overview with the Client ID:
![Image showing where the Client ID can be found](assets/Client%20ID.png)

And under that there is the button to reveal your client secret.

Next, go to `EDIT SETTINGS` on the overview page:
![Showing where the edit settings button is located](assets/Edit%20settings.png)

There add `https://localhost:8081` (The number after the `:` is the port and has to be the same that is set in the application itself) as a Redirect URI (Remember to click ADD):
![Showing how to add the redirect URI](assets/Add%20callback%20url.png)

### Installing

[Download](https://github.com/gaweringo/BOTM/releases) the exe from the releases page. And run it. It is a portable application, meaning it does not need to be installed, it will just open.

## 🎈 Usage <a name="usage"></a>

Fill out the Configuration, run `Generate BOTM` and follow the instructions. If everything works as intended you should see your BOTM playlist in a few seconds.

## ⛏️ Built Using <a name = "built_using"></a>

- [Rust](https://www.rust-lang.org/) - Programming language
- [rspotify](https://crates.io/crates/rspotify) - Spotify api crate
- [egui](https://crates.io/crates/egui) - GUI crate

## ✍️ Authors <a name = "authors"></a>

- [@gaweringo](https://github.com/gaweringo) - Putting the libs together

See also the list of [contributors](https://github.com/gaweringo/BOTM/contributors) who participated in this project.

## 🎉 Acknowledgements <a name = "acknowledgement"></a>

- Inspiration taken from [spotify-tui](https://github.com/Rigellute/spotify-tui/)
