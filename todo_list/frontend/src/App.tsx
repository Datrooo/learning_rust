import { useEffect, useMemo, useState } from "react";
import { TodoItem } from "./TodoItem";
import { TodoForm } from "./TodoForm";
import {
  createTodo,
  listTodos,
  updateTodo,
  deleteTodo,
  type Todo,
} from "./api";
import "./App.css";

export default function App() {
  const [todos, setTodos] = useState<Todo[]>([]);
  const [task, setTask] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  type Filter = "all" | "active" | "done";
  const [filter, setFilter] = useState<Filter>("all");
  const [theme, setTheme] = useState<"light" | "dark">(() => {
    const saved = localStorage.getItem("theme");
    return saved === "dark" ? "dark" : "light";
  });
  useEffect(() => {
    document.documentElement.dataset.theme = theme; // <html data-theme="...">
    localStorage.setItem("theme", theme);
  }, [theme]);

  const doneCount = useMemo(
    () => todos.filter((t) => t.is_finished).length,
    [todos]
  );

  const filteredTodos = useMemo(() => {
  if (filter === "active") return todos.filter((t) => !t.is_finished);
  if (filter === "done") return todos.filter((t) => t.is_finished);
  return todos;
  }, [todos, filter]);

  async function load() {
    try {
      setError(null);
      setLoading(true);
      const data = await listTodos();
      setTodos(data);
    } catch (e: any) {
      setError(e?.message ?? "Failed to load");
    } finally {
      setLoading(false);
    }
  }

  async function add() {
    const trimmed = task.trim();
    if (!trimmed) return;

    try {
      setError(null);
      const created = await createTodo({ task: trimmed });
      setTodos((prev) => [created, ...prev]);
      setTask("");
    } catch (e: any) {
      setError(e?.message ?? "Failed to create todo");
    }
  }

  async function toggle(t: Todo) {
    try {
      setError(null);
      const updated = await updateTodo(t.id, {
        is_finished: !t.is_finished,
      });
      setTodos((prev) => prev.map((x) => (x.id === updated.id ? updated : x)));
    } catch (e: any) {
      setError(e?.message ?? "Failed to update todo");
    }
  }

  async function remove(id: number) {
    try {
      setError(null);
      await deleteTodo(id);
      setTodos((prev) => prev.filter((t) => t.id !== id));
    } catch (e: any) {
      setError(e?.message ?? "Failed to delete todo");
    }
  }

  useEffect(() => {
    load();
  }, []);

  return (
    <div className="page">
      <div className="card">
        <div className="header">
          <h1 className="title">Todos</h1>

          <div className="headerRight">
            {!loading && (
              <p className="subtitle">
                Done: {doneCount}/{todos.length}
              </p>
            )}

          <label className="themeToggle" title="Toggle theme">
            <span className="subtitle">{theme === "dark" ? "Dark" : "Light"}</span>
              <span className="switch">
                <input
                  type="checkbox"
                  checked={theme === "dark"}
                  onChange={(e) => setTheme(e.target.checked ? "dark" : "light")}
                />
                <span className="slider" />
              </span>
            </label>
          </div>
        </div>

        {loading && <p className="subtitle">Loading...</p>}

        {error && (
          <div className="alert">
            <span>{error}</span>
            <button className="btn" onClick={load}>
              Retry
            </button>
          </div>
        )}

        <div className="filters">
          <button
            className={filter === "all" ? "pill pillActive" : "pill"}
            onClick={() => setFilter("all")}
          >
            All
          </button>
          <button
            className={filter === "active" ? "pill pillActive" : "pill"}
            onClick={() => setFilter("active")}
          >
            Active
          </button>
          <button
            className={filter === "done" ? "pill pillActive" : "pill"}
            onClick={() => setFilter("done")}
          >
            Done
          </button>
        </div>

        <TodoForm
          value={task}
          onChange={setTask}
          onSubmit={add}
          disabled={loading}
        />
      
        <ul className="list">
          {filteredTodos.map((t) => (
            <TodoItem key={t.id} todo={t} onToggle={toggle} onDelete={remove} />
          ))}
        </ul>
      </div>
    </div>
  );
}