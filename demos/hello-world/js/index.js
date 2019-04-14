import("../crate/pkg")
  .then(module => {
    module.run();
  })
  .catch(console.error);
