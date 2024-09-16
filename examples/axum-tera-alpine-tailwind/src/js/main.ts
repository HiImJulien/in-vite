import Alpine from "alpinejs";
import "./app.css";

Alpine.data("clicker", () => ({
  count: 0,

  click() {
    this.count += 1;
  }
}));

Alpine.start();

