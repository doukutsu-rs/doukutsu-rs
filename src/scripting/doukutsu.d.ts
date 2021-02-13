declare type EventHandler<T> = (this: void, param: T) => void;

/**
 * Represents a
 */
declare interface DoukutsuPlayer {
    /**
     * The ID of player.
     */
    id(): number;

    /**
     * Current position of player in X axis (as floating point, not internal fixed point representation).
     */
    x(): number;

    /**
     * Current position of player in Y axis (as floating point, not internal fixed point representation).
     */
    y(): number;

    /**
     * Current velocity of player in X axis (as floating point, not internal fixed point representation).
     */
    velX(): number;

    /**
     * Current velocity of player in Y axis (as floating point, not internal fixed point representation).
     */
    velY(): number;
}

declare interface DoukutsuScene {
    /**
     * Returns the tick of current scene.
     */
    tick(): number;

    /**
     * Returns a list of players connected to current game.
     */
    onlinePlayers(): DoukutsuPlayer[];

    /**
     * Returns a list of players on current map.
     */
    mapPlayers(): DoukutsuPlayer[];

    /**
     * Returns the id of local player.
     */
    localPlayerId(): number;

    /**
     * Returns player with specified id.
     */
    player(id: number): DoukutsuPlayer | null;
}

declare namespace doukutsu {
    /**
     * Plays a PixTone sound effect with specified ID.
     */
    function playSfx(id: number): void;

    /**
     * Changes current music to one with specified ID.
     * If ID equals 0, the music is stopped.
     */
    function playMusic(id: number): void;

    /**
     * Sets an implementation-defined game setting.
     * @param name
     * @param value
     */
    function setSetting(name: string, value: any): void;

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
    function on(event: "tick", handler: EventHandler<DoukutsuScene>): EventHandler<DoukutsuScene>;

    function on<T>(event: string, handler: EventHandler<T>): EventHandler<T>;
}
