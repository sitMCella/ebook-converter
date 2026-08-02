import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button } from '../ui/Button';
import { ConfirmDialog } from '../ui/ConfirmDialog';
import { ConversionQueue } from './ConversionQueue';
import { ConversionLog } from './ConversionLog';
import { CompletedList } from './CompletedList';
import { useConversionContext } from '../../contexts/ConversionContext';
import { useConversion } from '../../hooks/useConversion';

export function ConvertingScreen() {
  const navigate = useNavigate();
  const { state } = useConversionContext();
  const { cancelAll } = useConversion();
  const [showCancelDialog, setShowCancelDialog] = useState(false);

  const remainingCount = (state.activeFile ? 1 : 0) + state.queue.length;
  const hasActiveWork = state.activeFile != null || state.queue.length > 0;

  return (
    <div className="flex flex-col gap-5 p-6 max-w-3xl mx-auto">
      <div className="flex items-center justify-between">
        <h3 className="text-[16px] font-medium text-[var(--text-primary)]">
          {state.isComplete ? 'Conversion complete' : 'Converting'}
        </h3>
        {hasActiveWork && (
          <Button
            variant="secondary"
            onClick={() => setShowCancelDialog(true)}
          >
            Cancel all
          </Button>
        )}
        {state.isComplete && (
          <Button onClick={() => navigate('/converted')}>View converted</Button>
        )}
      </div>

      <ConversionQueue />
      <ConversionLog />
      <CompletedList />

      <ConfirmDialog
        open={showCancelDialog}
        title="Cancel conversions"
        message={`Cancel ${remainingCount} remaining conversion(s)? Files already converted are not affected.`}
        onConfirm={() => {
          cancelAll();
          setShowCancelDialog(false);
        }}
        onCancel={() => setShowCancelDialog(false)}
      />
    </div>
  );
}
