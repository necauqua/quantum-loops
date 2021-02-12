# Quantum Loops

![demo](https://necauqua.dev/images/level-3.gif)

This is a game I made for [Ludum Dare 47](https://ldjam.com/events/ludum-dare/47/quantum-loops)
in Rust, WASM and some pretty basic HTML5 canvas painting as well as some equally basic WebAudio.

The "engine" and the state management code that I wrote in Rust
are actually pretty remarkable in my opinion, given the jam time restraints.

The final LD47 commit is bf786dfe (well there is a tag also).
It is currently deployed at https://ld47.necauqua.dev, I never (yet) released/deployed
the updated code from the master branch, not to mention that the code from the master branch
contains unfinished changed at the moment.

This game is a pretty simple concept of an aim puzzle with some minimalist aesthetics
and calm background music.
However, I failed miserably to make the game describe what is the objective,
so if you really want to play it and failed to understand what you need to do from the vague
tutorial that it has, I recommend you to visit that LD47 link above and read the
step-by-step tutorial, and a straight description that I put there.

Maybe in the future updates I will make it better, however at the momend the development
of this project is frozen.

## Building
If you want to build this code yourself, make sure you have the basic Rust installation
going (Cargo), as well as that you have installed wasm-pack (with `cargo install wasm-pack`)
and NPM.

Then, you can follow what the `deploy.sh` file does:
- run `wasm-pack build`
- move into the NPM package directory: `cd www`
- run `npm run build` that will yield a `dist` directory,
  which you can serve as static files from something like Nginx
- alternatively, run `npm run start` if you want to run it immediately,
  then you can see the game at `localhost:3000`

## License
This project is licensed under the MIT license,
except the background music (`www/assets/background.mp3`),
It is Slow Motion by Bensound from the www.bensound.com, not owned by me by any means.
