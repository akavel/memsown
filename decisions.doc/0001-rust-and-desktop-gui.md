In the context of <!-- use case/story uc or component co -->
 the whole backerrs (a.k.a. MemsOwn) project,
facing <!-- concern c -->
 a question what programming language to write it in
 and whether to make it a standalone GUI app or a "Web GUI" app,

I decided to **<!-- option o -->
 write it in Rust as a standalone GUI app using the Iced library**,
and resigned from <!-- options o2 to oN -->:
 * Go - as I could not find a standalone GUI framework in it that I could use,
 * and the Go Web GUI frameworks I tried were missing many functionalities I needed,
 * nor could I find a stable embeddable database (I tried ql but it showed up to have showstopper integrity bug(s));
 * Nim - as I had a lot of problems with its stability and found documentation severely lacking
   when I tried to write a multi-threaded app,
   and also found myself too lost in my own code at some point due to Nim having looser typing discipline in common libraries vs. Rust,
 * libraries other than Iced as this one seemed most complete and understandable to me and actively developed
   (notably Druid was in an extremely (pre-)alpha-stage at the time of the decision making),
 * and Web GUIs tended to sooner or later require me to drop to HTML+CSS+JS anyway,
   which time and time again seems to just be too hard for me to get things done instead of becoming lost in confusion and failure;

to achieve <!-- quality q -->
 a possibility to write a cross-platform, multi-threaded app with a GUI,

accepting <!-- downside d -->
 slow compile times,
 amount of ceremony/LoC bloat in the language (Nim < Go < Rust < Java).

## Extra notes

- Rust, as well as Iced, should hopefully make a potential future addition of a mobile app relatively easiest
  (note esp. my https://github.com/akavel/dali project as one possible way to achieve this) -
  where Iced does not necessarily have a mobile backend yet,
  but provides a good and tested decoupling layer for backends,
  so adding a mobile one feels achievable.
- Rust seems to have a somewhat decent availability of various types of libraries as of now;
  notably *some* IPFS libs seem present, though not thoroughly reviewed by me yet.
  Still, IPFS feels like area of interest to Rust-minded people,
  so there's also some potential for improvement on this front (the later I will check, the better the situation should be).
- I wanted to learn Rust for a long time, and failing both in Go and Nim finally pushed me to try Rust -
  and I managed to get a lot further in it with the app (notably, GUI + SQLite + multithreading).
