There will come a day when I'll have to make the squares truly represent the pieces in them 
With pieces like the bus, this means also adding the passangers in the windows 
When I do that, I think I'll have to hard-code positions for the sub-pieces to be in? 
I'll also have to scale them down 
Then we get to nested pieces, like a bus carrying a goblin that has kidnapped a piece and such
But we'll deal with all that when we get there

Something like
```ts
type Anchor = {
  x: number; // 0–1 relative
  y: number; // 0–1 relative
  scale: number;
};

const pieceAnchors = {
  BUS: {
    windows: [
      { x: 0.25, y: 0.4, scale: 0.4 },
      { x: 0.5,  y: 0.4, scale: 0.4 },
      { x: 0.75, y: 0.4, scale: 0.4 },
    ],
    roof: { x: 0.5, y: 0.1, scale: 0.6 },
  },

  GOBLIN: {
    hands: { x: 0.6, y: 0.6, scale: 0.5 },
  },
};

```

Maybe?