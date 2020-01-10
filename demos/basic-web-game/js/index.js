import("../pkg/index.js")
  .then(mod => {
    const { WebScriptApi } = mod;

    class TestState {
      onEnter() {
        WebScriptApi.createEntity({
          'name': 'asshole',
          'age': { value: 42 },
        });
        WebScriptApi.createEntity({
          'name': 'asshole2',
        });
        WebScriptApi.createEntity({
          'age': { value: 43 },
        });
      }
    }
    WebScriptApi.registerStateFactory('main', () => new TestState());

    WebScriptApi.registerResource('globals', { hello: 'world' });

    WebScriptApi.registerComponentFactory('name', '');
    WebScriptApi.registerComponentFactory('age', { value: 0 });
    WebScriptApi.registerComponentFactory('data', { a: 4, b: 2 });
    WebScriptApi.registerComponentFactory('logic', { onProcess: () => console.log('=== PROCESS') });

    class GrowUpSystem {
      onRun() {
        const it = WebScriptApi.fetch(['&name', '&age']);
        while (it.next()) {
          // console.log(it.current());
        }
      }
    }
    WebScriptApi.registerSystem('grow-up', new GrowUpSystem());

    return WebScriptApi;
  })
  .then(WebScriptApi => WebScriptApi.markReady())
  .catch(console.error);
