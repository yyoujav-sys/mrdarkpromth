import React, { createContext, useContext, useState, useEffect } from 'react'
import { translations, type Language } from '@/lib/translations'

interface LanguageContextType {
    language: Language
    setLanguage: (lang: Language) => void
    t: (key: keyof typeof translations['en']) => string
}

const LanguageContext = createContext<LanguageContextType | undefined>(undefined)

export const LanguageProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const [language, setLanguage] = useState<Language>('en')

    useEffect(() => {
        const saved = localStorage.getItem('app_language') as Language
        if (saved) setLanguage(saved)
    }, [])

    const handleSetLanguage = (lang: Language) => {
        setLanguage(lang)
        localStorage.setItem('app_language', lang)
    }

    const t = (key: keyof typeof translations['en']) => {
        return translations[language][key] || key
    }

    return (
        <LanguageContext.Provider value={{ language, setLanguage: handleSetLanguage, t }}>
            {children}
        </LanguageContext.Provider>
    )
}

export const useLanguage = () => {
    const context = useContext(LanguageContext)
    if (context === undefined) {
        throw new Error('useLanguage must be used within a LanguageProvider')
    }
    return context
}
