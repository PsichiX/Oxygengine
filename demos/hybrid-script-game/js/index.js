import("../pkg/index.js")
  .then(mod => {
    const { WebScriptApi } = mod;

    WebScriptApi.registerResource('globals', {
      timeScale: 1,
    });

    WebScriptApi.registerComponentFactory('player', () => {
      return {};
    });

    WebScriptApi.registerComponentFactory('speed', () => {
      return { value: 1 };
    });

    class PlayerControlSystem {
      onRun() {
        const fetch = WebScriptApi.fetch(['+player', '+speed', '+CompositeTransform']);
        const globals = fetch.readResource('globals');
        const input = fetch.readResource('InputControllerState');
        const lifecycle = fetch.readResource('AppLifeCycle');
        if (input.triggers['mouse-left'] === 'Hold') {
          globals.timeScale = 0.3;
        } else {
          globals.timeScale = 1;
        }
        const dt = lifecycle.delta_time_seconds * globals.timeScale;
        const x = -input.axes['move-left'] + input.axes['move-right'];
        const y = -input.axes['move-up'] + input.axes['move-down'];
        while (fetch.next()) {
          const player = fetch.read(0);
          const speed = fetch.read(1).value;
          const transform = fetch.read(2);
          transform.translation.x += x * dt * speed;
          transform.translation.y += y * dt * speed;
          fetch.write(2, transform);
        }
      }
    }
    WebScriptApi.registerSystem('player-control', new PlayerControlSystem());

    class GameState {
      onEnter() {
        WebScriptApi.createEntity({
          CompositeCamera: {
            scaling: 'CenterAspect',
          },
          CompositeTransform: {
            scale: { x: 720, y: 720 },
          },
          AudioSource: {
            audio: 'ambient.ogg',
            streaming: true,
            play: true,
          },
        });

        WebScriptApi.createEntity({
          CompositeRenderable: {
            Rectangle: {
              color: { r: 128, g: 0, b: 0, a: 255 },
              rect: { x: -128, y: -128, w: 256, h: 256 },
            }
          },
          CompositeTransform: {
            translation: { x: 200, y: -100 },
            rotation: 45,
          },
        });

        WebScriptApi.createEntity({
          CompositeRenderable: {
            Image: {
              image: "logo.png",
              alignment: { x: 0.5, y: 0.5 },
            }
          },
          CompositeTransform: {},
          player: {},
          speed: { value: 100 },
        });
      }
    }
    WebScriptApi.registerStateFactory("main", () => new GameState());

    WebScriptApi.start();
  })
  .catch(console.error);
