import React from 'react';
import { useLanguage } from '@/contexts/LanguageContext';
import { Globe } from 'lucide-react';

export const LanguageSwitcher: React.FC = () => {
    const { language, setLanguage } = useLanguage();

    const toggleLanguage = () => {
        setLanguage(language === 'en' ? 'th' : 'en');
    };

    return (
        <button
            onClick={toggleLanguage}
            className="flex items-center gap-2 px-3 py-1.5 rounded-md bg-cyan-950/50 border border-cyan-500/30 
                       hover:border-cyan-400/50 hover:bg-cyan-900/50 transition-all duration-200
                       text-cyan-400 text-sm font-mono"
            title={language === 'en' ? 'Switch to Thai' : 'Switch to English'}
        >
            <Globe className="w-4 h-4" />
            <span className="uppercase">{language === 'en' ? 'EN' : 'TH'}</span>
        </button>
    );
};

export default LanguageSwitcher;
