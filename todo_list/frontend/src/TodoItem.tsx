import type { Todo } from "./api";

type Props = {
  todo: Todo;
  onToggle: (todo: Todo) => void;
  onDelete: (id: number) => void;
};

export function TodoItem({ todo, onToggle, onDelete }: Props) {
  return (
    <li className="item">
      <input
        className="checkbox"
        type="checkbox"
        checked={todo.is_finished}
        onChange={() => onToggle(todo)}
      />

      <span className={todo.is_finished ? "task taskDone" : "task"}>
        {todo.task}
      </span>

      <button className="btn" onClick={() => onDelete(todo.id)}>
        Delete
      </button>
    </li>
  );
}