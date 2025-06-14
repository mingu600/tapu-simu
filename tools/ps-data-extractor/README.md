# Pokemon Showdown Data Extractor

This tool extracts move and item data from Pokemon Showdown for use in Tapu Simu.

## Setup

```bash
npm install
```

## Usage

Extract all data:
```bash
npm run extract
```

Extract only moves:
```bash
npm run extract-moves
```

Extract only items:
```bash
npm run extract-items
```

## Output

The extracted data will be saved to:
- `../../data/ps-extracted/moves.json` - All move data
- `../../data/ps-extracted/items.json` - All item data

## Data Mapping

### Move Targets
Pokemon Showdown targets are mapped to Tapu Simu's position-based targets:
- `normal` → `SingleAdjacentTarget`
- `self` → `User`
- `adjacentAlly` → `SingleAlly`
- `allAdjacentFoes` → `AllAdjacentOpponents`
- etc.

### Move Categories
- `Physical` → `Physical`
- `Special` → `Special`
- `Status` → `Status`

## Notes

- Currently extracts Gen 9 data by default
- Filters out nonstandard moves/items (except Z-moves and Max moves)
- Includes all move flags, secondary effects, and special properties