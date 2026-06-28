import { render } from "solid-js/web";
import "./app.css";

function App() {
  return (
    <main class="min-h-screen flex items-center justify-center font-sans">
      <h1 class="text-3xl font-bold text-slate-900">url-shortener</h1>
    </main>
  );
}

const root = document.getElementById("root");
if (root === null) throw new Error("root element not found");
render(() => <App />, root);