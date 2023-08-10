declare type EventHandler<T> = (this: void, param: T) => void;

declare interface NPC {
    /**
     * The ID of NPC, equivalent to offset in NPC list of current scene.
     */
    id: number;

    /**
     * The type ID of NPC.
     */
    npcType: number;

    /**
     * Position of NPC in X axis (as floating point, not internal fixed point representation).
     */
    x: number;

    /**
     * Position of NPC in Y axis (as floating point, not internal fixed point representation).
     */
    y: number;

    /**
     * Velocity of NPC in X axis (as floating point, not internal fixed point representation).
     */
    velX: number;

    /**
     * Velocity of NPC in Y axis (as floating point, not internal fixed point representation).
     */
    velY: number;

    /**
     * Alternate velocity of NPC in X axis (as floating point, not internal fixed point representation).
     */
    velX2: number;

    /**
     * Alternate velocity of NPC in Y axis (as floating point, not internal fixed point representation).
     */
    velY2: number;

    /**
     * Current action id (the one that can be set with <ANP) of NPC (16-bit short integer).
     */
    actionNum: number;

    /**
     * First internal counter of the NPC (unsigned 16-bit short integer).
     */
    actionCounter: number;

    /**
     * Second internal counter of the NPC (unsigned 16-bit short integer).
     */
    actionCounter2: number;

    /**
     * Third internal counter of the NPC (unsigned 16-bit short integer).
     */
    actionCounter3: number;

    /**
     * Health of the NPC.
     */
    life: number;

    /**
     * Flag attached to this NPC.
     */
    flagNum: number;

    /**
     * Event attached to this NPC.
     */
    eventNum: number;

    /**
     * ID of the parent NPC.
     */
    parentId: number;

    /**
     * Direction of the NPC. Constrained to 0-4, use rawDirection to store values outside that range there.
     */
    direction: number;

    /**
     * Raw direction of the NPC, non-constrained and may have different value than direction field.
     * Used by certain vanilla NPCs that use the direction field for different purposes.
     */
    rawDirection: number;

    hitCeiling(): boolean;

    hitFloor(): boolean;

    hitLeftWall(): boolean;

    hitRightWall(): boolean;

    /**
     * Pick a random integer from given range using RNG bound to this NPC.
     * @param min minimum value.
     * @param max maximum value.
     */
    random(min: number, max: number): number;

    /**
     * Returns a reference to parent NPC, if present.
     */
    parentNPC(): NPC | null;

    /**
     * Returns a reference to closest player, non-nullable.
     */
    closestPlayer(): DoukutsuPlayer;

    /**
     * Internal counter used as index of animation frame. (unsigned 16-bit short integer)
     */
    animNum: number;

    /**
     * Internal counter used by animation functions to count ticks between frames (unsigned 16-bit short integer).
     */
    animCounter: number;

    /**
     * Returns sprite bounds of the NPC.
     */
    getAnimRect(): [number, number, number, number];

    /**
     * Sets the sprite bounds to specified rectangle.
     * @param rect [left, top, right, bottom] rectangle.
     */
    setAnimRect(rect: [number, number, number, number]): void;

    /**
     * Sets the sprite bounds to specified rectangle.
     * @param left left bound
     * @param top top bound
     * @param right right bound
     * @param bottom bottom bound
     */
    setAnimRect(left: number, top: number, right: number, bottom: number): void;
}

/**
 * Represents an in-game player.
 */
declare interface DoukutsuPlayer {
    /**
     * The ID of player.
     */
    id: number;

    /**
     * Current position of player in X axis (as floating point, not internal fixed point representation).
     */
    x: number;

    /**
     * Current position of player in Y axis (as floating point, not internal fixed point representation).
     */
    y: number;

    /**
     * Current velocity of player in X axis (as floating point, not internal fixed point representation).
     */
    velX: number;

    /**
     * Current velocity of player in Y axis (as floating point, not internal fixed point representation).
     */
    velY: number;

    /**
     * Damages the player. Has no effect when invincibility is enabled.
     * @param value number of health points to subtract.
     */
    damage(value: number): void;
}

declare interface DoukutsuRSApi {
    /**
     * Lighting mode of current stage.
     * "none" - no lighting, similar to vanilla.
     * "backgroundOnly" - lighting only affects background layer, similar to Switch version.
     * "ambient" - lighting affects everything.
     */
    lightingMode: "none" | "backgroundOnly" | "ambient";

    /**
     * This property is true if lighting is enabled in settings.
     */
    readonly lightingEnabled: boolean;
}

declare namespace doukutsu {
    /**
     * A reference to main locally controlled player.
     * In multiplayer context it refers to player who hosts the game.
     */
    const player: DoukutsuPlayer;

    /**
     * Helper property for doukutsu-rs specific APIs.
     */
    const rs: DoukutsuRSApi;

    /**
     * The number of current stage, read-only. Set to -1 if in menu.
     */
    const currentStage: number;

    /**
     * Plays a sound effect with specified ID.
     */
    function playSfx(id: number): void;

    /**
     * Plays a looping sound effect with specified ID.
     */
    function playSfxLoop(id: number): void;

    /**
     * Changes current music to one with specified ID.
     * If ID equals 0, the music is stopped.
     * If ID equals 0 and fadeout is true, the music is faded out.
     */
    function playMusic(id: number, fadeout: boolean = false): void;

    /**
     * Returns the value of a certain TSC flag.
     * @param id the flag number
     */
    function getFlag(id: number): boolean;

    /**
     * Sets the value of a certain TSC flag.
     * @param id the flag number
     * * @param value the flag value
     */
    function setFlag(id: number, value: boolean): void;

    /**
     * Returns the value of a certain skip flag.
     * @param id the flag number
     */
    function getSkipFlag(id: number): boolean;

    /**
     * Sets the value of a certain skip flag.
     * @param id the flag number
     * @param value the flag value
     */
    function setSkipFlag(id: number, value: boolean): void;

    /**
     * Returns a list of players currently in game.
     */
    function players(): DoukutsuPlayer[];

    /**
     * Returns a reference to NPC by it's ID (index in table).s
     * @param id
     */
    function getNPC(id: number): NPC;

    /**
     * Sets an implementation-defined game setting.
     * @param name
     * @param value
     */
    function setSetting(name: string, value: any): void;

    /**
     * Sets an implementation-defined stage parameter.
     * @param name
     * @param value
     */
    function setStageParam(name: string, value: any): void;

    /**
     * Sets the handler override for specified NPC type. Passing a null removes the handler.
     * @param npcType
     * @param handler
     */
    function setNPCHandler(npcType: number, handler: (this: void, npc: NPC) => void | null): void;

    /**
     * Registers an event handler called after all scripts are loaded.
     * @param event event name
     * @param handler event handler procedure
     */
    function on(event: "init", handler: EventHandler<void>): EventHandler<void>;

    /**
     * Registers an event handler called on each tick.
     * @param event event name
     * @param handler event handler procedure
     */
    function on(event: "tick", handler: EventHandler<DoukutsuStage>): EventHandler<DoukutsuStage>;

    function on<T>(event: string, handler: EventHandler<T>): EventHandler<T>;
}
