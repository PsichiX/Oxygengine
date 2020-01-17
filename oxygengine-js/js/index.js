import('../pkg/oxygengine.js')
  .then(mod => {
    const { WebScriptApi } = mod;

    WebScriptApi.registerResource('globals', {
      timeScale: 1,
      ready: false,
    });

    WebScriptApi.registerComponentFactory('speed', () => {
      return { value: 1 };
    });

    WebScriptApi.registerComponentFactory('player', () => {
      return {};
    });


    WebScriptApi.registerComponentFactory('loader', () => {
      return {};
    });

    class LoadingControlSystem {
      constructor() {
        this.phase = 0;
      }

      onRun() {
        const fetch = WebScriptApi.fetch(['+loader', '+CompositeTransform']);
        const globals = fetch.readResource('globals');
        const lifecycle = fetch.readResource('AppLifeCycle');
        this.phase += lifecycle.delta_time_seconds;

        while (fetch.next()) {
          const transform = fetch.read(1);
          if (globals.ready) {
            const s = 1 + Math.sin(this.phase * Math.PI) * 0.4;
            transform.scale.x = s;
            transform.scale.y = s;
            transform.rotation = 45;
          } else {
            transform.scale.x = 1;
            transform.scale.y = 1;
            transform.rotation = Math.sin(this.phase * Math.PI) * 180;
          }
          fetch.write(1, transform);
        }
      }
    }
    WebScriptApi.registerSystem('loading-control', new LoadingControlSystem());

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
          const speed = fetch.read(1).value;
          const transform = fetch.read(2);
          transform.translation.x += x * dt * speed;
          transform.translation.y += y * dt * speed;
          fetch.write(2, transform);
        }
      }
    }
    WebScriptApi.registerSystem('player-control', new PlayerControlSystem());

    class LoadingState {
      onEnter() {
        const fetch = WebScriptApi.fetch([]);
        const token = fetch.readResource('AppLifeCycle').current_state_token;

        WebScriptApi.createEntity({
          CompositeCamera: {
            scaling: 'CenterAspect',
          },
          CompositeTransform: {
            scale: { x: 720, y: 720 },
          },
          NonPersistent: token,
        });

        WebScriptApi.createEntity({
          CompositeRenderable: {
            Rectangle: {
              color: { r: 255, g: 255, b: 0, a: 255 },
              rect: { x: -32, y: -32, w: 64, h: 64 },
            }
          },
          CompositeTransform: {},
          NonPersistent: token,
          loader: {},
        });

        fetch.writeResource('InputControllerMappings', {
          mapping_axes: {
            'move-up': ['keyboard', 'KeyW'],
            'move-down': ['keyboard', 'KeyS'],
            'move-left': ['keyboard', 'KeyA'],
            'move-right': ['keyboard', 'KeyD'],
          },
          mapping_triggers: {
            'mouse-left': ['mouse', 'left'],
          },
        });

        fetch.accessResource(
          'AssetsDatabase',
          { load: ['pack://assets.pack'] }
        );

        this.phase = 0;
      }

      onProcess() {
        if (this.phase === 0) {
          this.processLoadingPack();
        } else if (this.phase === 1 && this.processLoadingSet()) {
          return 'game';
        }
      }

      processLoadingPack() {
        const fetch = WebScriptApi.fetch([]);
        const loaded = fetch.accessResource(
          'AssetsDatabase',
          { loaded: ['pack://assets.pack'] }
        );

        if (!!loaded && loaded['pack://assets.pack']) {
          fetch.accessResource(
            'AssetsDatabase',
            { 'serve-pack': 'pack://assets.pack' }
          );

          fetch.accessResource(
            'AssetsDatabase',
            { load: ['set://assets.txt'] }
          );

          this.phase = 1;
        }
      }

      processLoadingSet() {
        const fetch = WebScriptApi.fetch([]);
        const loaded = fetch.accessResource(
          'AssetsDatabase',
          { loaded: ['set://assets.txt'] }
        );

        if (!!loaded && loaded['set://assets.txt']) {
          fetch.readResource('globals').ready = true;

          const input = fetch.readResource('InputControllerState');
          if (input.triggers['mouse-left'] === 'Pressed') {
            return true;
          }
        }

        return false;
      }
    }
    WebScriptApi.registerStateFactory('main', () => new LoadingState());

    class GameState {
      onEnter() {
        const fetch = WebScriptApi.fetch([]);
        const token = fetch.readResource('AppLifeCycle').current_state_token;

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
          NonPersistent: token,
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
          NonPersistent: token,
        });

        WebScriptApi.createEntity({
          CompositeRenderable: {
            Image: {
              image: 'logo.png',
              alignment: { x: 0.5, y: 0.5 },
            }
          },
          CompositeTransform: {},
          NonPersistent: token,
          player: {},
          speed: { value: 100 },
        });
      }
    }
    WebScriptApi.registerStateFactory('game', () => new GameState());

    WebScriptApi.start();
  })
  .catch(console.error);
