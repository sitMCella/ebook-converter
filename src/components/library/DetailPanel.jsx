import { useNavigate } from 'react-router-dom';
import { ArrowRightLeft, RefreshCw } from 'lucide-react';
import { useConversion } from '../../hooks/useConversion';
import { PagePreview } from './PagePreview';
import { MetadataSection } from './MetadataSection';
import { ConversionOptions } from './ConversionOptions';
import { Button } from '../ui/Button';

export function DetailPanel({ file }) {
  const navigate = useNavigate();
  const { startConversion } = useConversion();

  const isConverting = file.status === 'converting';
  const isConverted = file.status === 'converted';

  const handleConvert = () => {
    startConversion([file.path]);
    navigate('/converting');
  };

  return (
    <div className="p-5 flex flex-col gap-6">
      <PagePreview file={file} />
      <MetadataSection file={file} />
      <ConversionOptions file={file} />

      <Button
        variant={isConverted ? 'secondary' : 'primary'}
        disabled={isConverting}
        onClick={handleConvert}
      >
        {isConverted ? (
          <>
            <RefreshCw size={16} />
            Reconvert to EPUB
          </>
        ) : (
          <>
            <ArrowRightLeft size={16} />
            Convert to EPUB
          </>
        )}
      </Button>
    </div>
  );
}
