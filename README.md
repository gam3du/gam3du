# Gam3du

> [!NOTE]
> 🇩🇪 [Deutschsprachige Fassung dieses Dokumentes](README.de.md)

A game engine written for educational purposes. The main goals are learning/teaching software development, ease of use and having fun!

> In the beginning, there was `0x00`.

This project is currently in its proof-of-concept phase, where we try to evaluate whether the undertaking is feasible as a whole. Next goal is an MVP (minimum viable product) that can be used to teach simple programming tasks in the Python programming language.

![Screenshot of the current prototype, where a Python-controlled robot draws lines and paints tiles on a plane](documentation/robot-screenshot.png)

## Motivation

If you asked Joe Average how many computers he possesses, he likely answer something between "none" to maybe "two". Reminding Joe that everything with a display is a computer as well, he might come up with some additional smartphones, smartwatches, tablets and TVs.

The reality is: Joe very likely owns way more computers than lamps (not even counting the computers within the lamps). Computers are everywhere: TV-remote, coffee maker, car stereo, calculator, Bluetooth headset, door bell, camera, electrical toothbrush, printer, washing machine, … as a rule of thumb: when it runs on electrical energy it likely contains a computer nowadays. Someone has to tell all those little helpers what to do when a button is pressed, the battery runs low or the wireless connections drops out.

This _someone_ is a software developer - often casually called a _programmer_. As developers don't grow on [trees](https://en.wikipedia.org/wiki/Tree_(data_structure)), we need to resort to a more classic approach: _education_. While this sounds like a downer at first, most good developers [I](https://github.com/kawogi) know, are enthusiastic about tech-stuff in general: give them something interesting to tinker with and they'll give you something impressive back.

Nowadays schools are starting to teach programming classes from the 5th grade at the age of 10 (at least in Germany). One great resource is [Scratch](https://scratch.mit.edu/). Scratch lets you build little games and animations by visually arranging command blocks. Other platforms like [Jugendwettbewerb Informatik](https://jwinf.de/) provide a series of programming challenges of increasing difficulty which can be solved by either using the visual Scratch-approach or using the Python programming language. This is a nice transition towards a textual programming language, which is the industry norm.

Ok, the 6th grade is over. You know that programming is something you might like and your English skills are good enough to understand the few required keywords of a real programming language. Now what?

The usual approach is to go back to your grandfather's terminal and learn the programming basics one by one with a series of simple examples. Trying to get your code right within 30 minutes of trial-and-error just to finally output the awaited result `42` in a text box isn't exactly everyone's understanding of "fun with computers".

Wouldn't it be great to just continue where you left off? With Scratch or the line-drawing turtle you already had something game-like under your control. Why not just take this to the next level? But where to start?

This is the gap we're trying to fill.

## What we do

We're writing our own game engine.

Uh, wait … that sounds ambitious and there are numerous game engines out there, so why write another one?

Most game engines are meant to churn out commercially successful games, so they focus on re-using existing solutions. Re-inventing the wheel (or rails, jet-packs, teleporters, …) would unnecessarily defer the release date. Also, successful game engines are often commercially targeted and may not be affordable for schools and students. Another reason is that existing game engines are often closed source, so if you want to dig deeper than writing the nth iteration of a platformer, shooter or puzzle game, you'll need to look elsewhere.

Another important reason to write a game engine: it's challenging. In other words: fun!

_Gam3du_ won't be the fastest game engine on the market. Also it won't be the most versatile and not even deliver the most impressive graphics. So, what's left? _Gam3du_ aims to be:

- _approachable_ - it shall always be easy to "just start coding". No overly complicated installation procedures, system dependencies, build-scripts, package managers, …
- _simple_ - we value readability over complex solutions. There's still a balance to uphold when it comes to performance and resource consumption.
- _extensible_ - the engine will only cover the core elements of your game. Everything else will be mods written by the community!
- _robust_ - The engine has to deal with a lot of user code which isn't always _as optimal as it could be_. It's our job to not let things get out of control in those cases.
- _debuggable_ - Things will still break and when they do, you deserve a good explanation what went wrong. Extensive logs and context information shall make it easier to get you back on track.

## Target audience

> How many people do you need to [add a door to your game](https://lizengland.com/blog/2014/04/the-door-problem/)?

_Gam3du_ is meant to serve a wide range of educational and creative purposes.

- Teaching: Teachers can use this platform to provide their students with programming tasks of varying complexity. There will be a choice of common tasks but new ones can be created and shared as well.
- Game programming with scripts: Students may use the platform to create simple games or simulations in a 3D-environment.
- Creative: creating 3D-models, sounds, textures, shader programming (WGSL) …
- Working on the game engine: This is for experienced developers who want to extend the abilities of the platform as a whole

## Challenges

### Time

This is the main problem with all non-commercially driven projects: finding enough volunteers having the right skills with enough free time to work on something together. This project is about education and as such cannot expect to get any money to compensate for the time invested here.

### Language barrier

At the moment our small team is entirely located in Germany. Using English as the primary communication language isn't as common as in other (smaller) countries. We'll try to make up for this by providing localized front-ends for the entry-level programming tasks.

## How you can help

At the moment we're still trying to get our act together. We'll update this section as soon as the fog lifts.

## How to build and run

1. [Install the Rust toolchain](https://www.rust-lang.org/learn/get-started)
2. Clone this repository: `git clone git@github.com:gam3du/gam3du.git`
3. Change into the project directory: `cd gam3du`
4. Build and run the code: `cargo run --bin=launcher-robot`

This will open a new window, showing a plane where a Python-controlled robot is moving across the grid.

Exit with `ESC` or just close the window.

## Further reads

Open Source game engines and renderers:

- [OGRE](https://www.ogre3d.org/) - rendering engine written in C++
- [GODOT](https://godotengine.org/) - game engine written in C++
- [BEVY](https://bevyengine.org/) - game engine written in Rust

Programming languages:

- [Python](https://www.python.org/) - popular, versatile and easy to learn; often taught at schools
- [Lua](https://www.lua.org/) - very simple scripting language; used by several popular games
- [Java](https://www.java.com/en/) - very popular object-oriented programming language; often used in academic environments and serious businesses.
- [Rust](https://www.rust-lang.org/) - safe, high-performance system level language; prior programming experience is recommended (_Gam3du_ is written in Rust)
- [C++](https://isocpp.org/) - extremely versatile system-level language; most popular game engines and native programs are written in C++; requires a lot of discipline

Game development:

- [Game Programming Patterns](https://gameprogrammingpatterns.com/) - Book by Robert Nystrom; this is a must-read if you're seriously into game programming.
- [Game development in Rust](https://arewegameyet.rs/) - Are we game yet? - Almost. We have the blocks, bring your own glue.

## Who we are

Not many so far. We're a small team of students, teachers and IT-professionals working on this project in their free-time.
