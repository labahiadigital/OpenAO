export type EditorTool = "select" | "paint" | "erase" | "fill" | "block" | "npc" | "spawn" | "tp";

export type EditorLayer = "ground" | "objects" | "roofs" | "triggers" | "blocked";

type TileEdit = {
  map: number;
  x: number;
  y: number;
  layer: EditorLayer;
  oldValue: number;
  newValue: number;
};

class EditorStore {
  active = $state(false);
  currentTool: EditorTool = $state("select");
  currentLayer: EditorLayer = $state("ground");
  selectedTileId = $state(0);
  mapId = $state(1);
  isDirty = $state(false);

  private undoStack = $state<TileEdit[][]>([]);
  private redoStack = $state<TileEdit[][]>([]);
  private pendingBatch = $state<TileEdit[]>([]);

  setTool(tool: EditorTool) {
    this.currentTool = tool;
  }

  setLayer(layer: EditorLayer) {
    this.currentLayer = layer;
  }

  selectTile(tileId: number) {
    this.selectedTileId = tileId;
  }

  startEditBatch() {
    this.pendingBatch = [];
  }

  addEdit(edit: TileEdit) {
    this.pendingBatch.push(edit);
  }

  commitEditBatch() {
    if (this.pendingBatch.length > 0) {
      this.undoStack.push([...this.pendingBatch]);
      this.redoStack = [];
      this.pendingBatch = [];
      this.isDirty = true;
    }
  }

  undo() {
    const batch = this.undoStack.pop();
    if (batch) {
      this.redoStack.push(batch);
      this.isDirty = this.undoStack.length > 0;
    }
  }

  redo() {
    const batch = this.redoStack.pop();
    if (batch) {
      this.undoStack.push(batch);
      this.isDirty = true;
    }
  }

  get canUndo(): boolean {
    return this.undoStack.length > 0;
  }

  get canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  reset() {
    this.undoStack = [];
    this.redoStack = [];
    this.pendingBatch = [];
    this.isDirty = false;
  }
}

export const editorStore = new EditorStore();
