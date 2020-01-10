import("../pkg/index.js")
  .then(mod => {
    const { WebScriptApi } = mod;

    WebScriptApi.registerResource('globals', { hello: 'world' });

    WebScriptApi.registerComponentFactory('name', () => {
      return { value: '' };
    });

    WebScriptApi.registerComponentFactory('age', () => {
      return { value: 0 };
    });

    class GrowUpSystem {
      onRun() {
        const it = WebScriptApi.fetch(['@', '$globals', '&name', '&age']);
        while (it.next()) {
          const [entity, globals, name, age] = it.current();
          age.value += 1;
          console.log('===', entity, globals, name, age);
          if (age.value > 100) {
            WebScriptApi.destroyEntity(entity);
          }
        }
      }
    }
    WebScriptApi.registerSystem('grow-up', new GrowUpSystem());

    class TestState {
      onEnter() {
        WebScriptApi.createEntity({
          'name': { value: 'asshole' },
          'age': { value: 42 },
        });
        WebScriptApi.createEntity({
          'name': { value: 'asshole2' },
        });
        WebScriptApi.createEntity({
          'age': { value: 43 },
        });
      }
    }
    WebScriptApi.registerStateFactory('main', () => new TestState());

    return WebScriptApi;
  })
  .then(WebScriptApi => WebScriptApi.markReady())
  .catch(console.error);
