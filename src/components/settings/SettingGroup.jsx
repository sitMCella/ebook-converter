export function SettingGroup({ title, children }) {
  return (
    <div className="flex flex-col gap-0">
      <h4 className="text-[14px] font-medium pb-2 mb-2 border-b border-[var(--border)]" style={{ borderBottomWidth: '0.5px' }}>
        {title}
      </h4>
      <div className="flex flex-col">
        {children}
      </div>
    </div>
  );
}
