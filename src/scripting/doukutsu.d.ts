declare type EventHandler<T> = (this: void, param: T) => void;

declare interface DoukutsuPlayer {
    x(): number;
    y(): number;
    velX(): number;
    velY(): number;
};

declare interface DoukutsuScene {
    /**
     * Returns the tick of current scene.
     */
    tick(): number;

    /**
     * Returns player at specified index.
     */
    player(index: number): DoukutsuPlayer | null;
};

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

    function on(event: "tick", handler: EventHandler<DoukutsuScene>): EventHandler<DoukutsuScene>;

    function on<T>(event: string, handler: EventHandler<T>): EventHandler<T>;
};
