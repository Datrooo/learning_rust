type Props = {
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  disabled?: boolean;
};

export function TodoForm({ value, onChange, onSubmit, disabled }: Props) {
  return (
    <div className="form">
      <input
        className="input"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Новая задача..."
        maxLength={500}
        disabled={disabled}
        onKeyDown={(e) => {
          if (e.key === "Enter") onSubmit();
        }}
      />
      <button
        className="btn btnPrimary"
        onClick={onSubmit}
        disabled={disabled}
      >
        Add
      </button>
    </div>
  );
}