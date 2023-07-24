# imvr
An image viewer for all platforms, allowing for remote control. Designed to be a terminal image program that isn't a band-aid like ueberzugg but rather an alternative option to trying to patch something into a 40 year old technology. It is primarily designed to embedable in applications but also should be great for just personal use and scripting.

## TODO
- [ ] parser thread that pre-process the image so the render thread is responsive
- [ ] resizing window to fit image size
- [ ] removing more bad code
- [ ] abstracting commands from arbitrary sources:
    - [ ] stdin
    - [ ] socket
    - [ ] window key presses
    - [ ] terminal tui
    - [ ] cli (that connects to socket)
