# Modal presenter

Like to do presentations with live-coding? Then you know that although it's a great way to present tech, it's pretty error prone!

I build this tool so that I can play back the live coding from a scenario file, and use a NES joystick as a presenter for both my slides and to have my laptop type stuff.

## Idea
- I use i3. I'll put my presentation on workspace 1, and my live demo on workspace 2.
- I can switch with `Mod+{1,2}` to respetively workspace 1 & 2.
- Catching the joystick events with SDL, and use libxdo do send keystrokes to my X-server.
- Have a minimal UI to show the mode (Presentation, or Demo), a timer, and the next command that will run in the demo mode.
