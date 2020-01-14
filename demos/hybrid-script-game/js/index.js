import("../pkg/index.js")
  .then(mod => {
    const { WebScriptApi } = mod;

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
        });
      }
    }
    WebScriptApi.registerStateFactory("main", () => new GameState());

    WebScriptApi.start();
  })
  .catch(console.error);
