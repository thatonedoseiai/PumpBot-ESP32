# Screen drawing ideas

There are many ways in which we could potentially be drawing to the screen.
1. Direct draw and undraw
2. Difference redraw with double buffering
3. ???

We will go through each of these possibilities step-by-step.

## 1. Direct Draw

Under this method, drawing a string of text would produce a structure `Sprite`. On creation, `Sprite` would buffer a draw to the screen. In order to facilitate redrawing when new text is drawn, `Sprite` would have a custom `Drop<'a>` implementation. This Drop implementation will draw a blank rectangle to the screen. 

This is akin to what we have already in bitmap-font-rendering.

IMPLEMENTATION TODOS:
- [ ] create a `sprite` struct on drawing
- [ ] implement the `Drop` trait manually

## 2. Difference redraw with double buffering

Under this method, we keep two buffers. Drawing a string of text would create multiple `Sprite`s, one for each character. Each character would also produce a bounding box which would be stored in the "onscreen buffer" (OSB).

Any time we queue draws to the screen while certain characters are marked for undrawing, we check the `OSB` and see what sprites it intersects. If the sprite intersects another sprite to be overdrawn, the queued overdraw action will be cropped to reduce repeated SPI bus draw calls.

- [ ] create a `sprite` struct on drawing
- [ ] Find a way to safely store a reference to the bounding box whenever we draw