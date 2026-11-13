import type { Metadata } from 'next';
import DeviceClient from './DeviceClient';
import { getTranslator } from '@/lib/i18n/server';

export async function generateMetadata(): Promise<Metadata> {
  const { t } = await getTranslator();

  return {
    title: t('settings.pairing.meta.title'),
    description: t('settings.pairing.meta.description'),
  };
}

export default function DevicePage() {
  return <DeviceClient />;
}
