# Bevy Jam 8

This project was generated using the [Bevy New 2D](https://github.com/TheBevyFlock/bevy_new_2d) template.
Check out the [documentation](https://github.com/TheBevyFlock/bevy_new_2d/blob/main/README.md) to get started!

# Design Doc

The game will be primarily a platformer, with a secondary collection and crafting component. The general process will be:
    - Easy platforming to collect materials and combine in a cauldron into potions
    - Hard platforming to collect aether using potions and player skill
    - Easy platforming to use aether and materials in a cauldron to upgrade inventory or create gold
    - Loop above until you reach enough gold, then race to retirement through a hard platforming section up a mountain
    - Reach a small cabin at the top of the mountain, retire, and roll credits

## Phases
The gameplay loop will have three major phases, with varying intensity platforming in each phase:
1) A resource buildup phase, with 3 subphases:
    - A material collection and prep phase, which is slow and planned. Here they'll 
    - Once the player has their layout prepped, they will enter a fast-paced segment where they try to stay afloat inside a difficult area for as long as possible against wind, gravity, and possibly hazards. There they'll collect a critical material (aether?) for creating gold.
    - They'll combine that critical material with other materials to create gold at the cauldron.
2) A race towards retirement phase. When the player meets a gold threshold, they'll be pointed towards the top of a nearby mountain. This will require both collection and platforming execution to complete.
3) Retirement will be in a small cabin on the top of the mountain, where you'll sit in a chair with an elated variant of the earlier music.

## Map
- Four main sections, each with a different play feel
    - Easy beginning: easy platforming, mild challenge, slow pace
    - Difficult middle: difficult section, sparse platforms, includes hazards like directional wind
    - Mixed mountain: a combination of everything from the first two, functions as the final stretch of the game
    - Retirement: no platforming or movement, just satisfied sitting with great music and credits
- Maybe each section can be directly above the previous. Particularly for the first two, 
- Play feel will be defined by
    - Density of materials
    - Placement of platforms
    - Density of hazards, like directional wind
- Will have cauldrons and materials placed around the map in set locations which respawn as necessary
- Some aesthetic change between the areas, including music and visuals, but not sure how much will be practical

## Cauldrons
- Will allow you to combine materials into potions
- Spread around map in manually placed locations
- Should have a combining UI, which allows the players to select materials and then hit combine
    - I don't have an exact idea on the UI yet

## Gold
- A main resource for the finishing the game
    - No use otherwise, unless we want to do upgrades through a shop instead of the cauldron but that's more work for not very much payoff so it's a stretch goal.
- When you reach the set gold threshold, allow the player to do the final mountain segment to retire and win

## Aether
- Aether is a special material which is a bottleneck to creating gold to win the game
- Can only be gotten in the difficult middle area of the map

## Materials
- 3 different materials
    - MVP just make them different shapes/colors
        - Verdant - currently a green square
        - Lavender - currently a purple triangle
        - Cerulean - currently a blue circle
    - (Stretch) Give them some better visuals and effects on pickup or use or respawn
- Spawned across the map in pre-determined locations
- Can be collected by the player
- Will respawn in place on a timer
- When added to inventory, those materials can be combined into different potions at cauldrons

## Potions
- Potions will be created by combining materials into a cauldron on the map
- Potions will be consumed to activate a movement ability or buff
- (Stretch) Each potion will have three levels of strength
- Planned potions, but extremely open to suggestions:
    - Horizontal dash (1 dash per strength level per potion, probably?)
    - Featherweight (recieve more intense wind effect for 1 s per strength level?)
    - Air jump (distance increased per strength?)

## Inventory
- Inventory will be owned by the player and have:
    - Items
        - Materials
        - Potions
        - Aether
        - Gold
    - Properties
        - Max materials
        - Max potions
        - (Stretch) mixing properties like potion strength 
- Inventory should also have its own UI elements to communicate status of current materials, potions, aether and gold at a glance

## Aesthetics and vibes
- Vibes start uncertain/somber/tense then move towards elation/soaring/freedom as you approach retirement
- This really isn't my area so it would be great if someone would take the lead here
- Aesthetics in a wide sense, including overall visuals/sounds/effects style and any specific visual/sound/effect
