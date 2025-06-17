# Pokemon Showdown Data Extractor

This tool extracts move and item data from Pokemon Showdown for use in Tapu Simu.

## Setup

```bash
npm install
```

## Usage

### Basic Extraction

Extract all data (Gen 9):
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

### Generation-Specific Extraction

Extract generation-specific data with change analysis:
```bash
node extract-generations.js all          # Extract both moves and items across all generations
node extract-generations.js moves        # Extract only moves across all generations
node extract-generations.js items        # Extract only items across all generations
```

## Output

### Basic Extraction
The extracted data will be saved to:
- `../../data/ps-extracted/moves.json` - All move data (Gen 9)
- `../../data/ps-extracted/items.json` - All item data (Gen 9)
- `../../data/ps-extracted/pokemon.json` - All Pokemon data (Gen 9)

### Generation-Specific Extraction
- `../../data/ps-extracted/moves-by-generation.json` - Move data for each generation
- `../../data/ps-extracted/move-changes.json` - Analysis of move changes across generations
- `../../data/ps-extracted/items-by-generation.json` - Item data for each generation
- `../../data/ps-extracted/item-changes.json` - Analysis of item changes across generations

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