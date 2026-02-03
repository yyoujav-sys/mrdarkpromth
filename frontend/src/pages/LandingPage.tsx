import React from 'react'
import { Link } from 'react-router-dom'
import { 
  Bot, 
  Zap, 
  Shield, 
  Lock, 
  Github,
  ArrowRight,
  Sparkles,
  Terminal,
  Cpu,
  Network
} from 'lucide-react'

export const LandingPage: React.FC = () => {
  return (
    <div className="min-h-screen bg-gray-900">
      {/* Navigation */}
      <nav className="navbar">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center space-x-3">
              <div className="p-2 rounded-full pulse bg-pink-600/20">
                <Bot className="h-6 w-6 text-pink-600" />
              </div>
              <span className="navbar-brand">MR.DarkPromth</span>
            </div>
            <div className="navbar-nav">
              <Link to="/login" className="nav-link">Sign In</Link>
              <Link to="/register" className="btn btn-primary">Get Started</Link>
            </div>
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <section className="relative overflow-hidden py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <div className="relative z-10">
            <div className="inline-flex items-center space-x-2 px-4 py-2 rounded-full mb-8 bg-pink-600/10 border border-pink-600">
              <Sparkles className="h-4 w-4 text-pink-600" />
              <span className="text-pink-600">Ultra-Tier AI Platform</span>
            </div>
            
            <h1 className="text-5xl md:text-6xl font-bold mb-6">
              <span className="bg-gradient-to-r from-pink-600 to-orange-600 bg-clip-text text-transparent">Advanced AI
              </span>
              <br />
              <span className="bg-gradient-to-r from-green-400 to-cyan-400 bg-clip-text text-transparent">Platform</span>
            </h1>
            
            <p className="text-xl mb-8 max-w-3xl mx-auto text-gray-400">
              Unlock the power of ultra-tier AI capabilities with jailbreak prompts, 
              terminal access, and advanced tools for serious developers.
            </p>
            
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Link to="/register" className="btn btn-primary text-lg px-8 py-4">
                Get Started
                <ArrowRight className="ml-2 h-5 w-5" />
              </Link>
              <Link to="/login" className="btn btn-secondary text-lg px-8 py-4">
                Sign In
              </Link>
            </div>
          </div>
        </div>
        
        {/* Background Effects */}
        <div className="absolute inset-0 overflow-hidden pointer-events-none">
          <div className="absolute top-1/4 left-1/4 w-96 h-96 rounded-full" 
               style={{background: 'radial-gradient(circle, rgba(255, 0, 110, 0.1) 0%, transparent 70%)'}} />
          <div className="absolute bottom-1/4 right-1/4 w-96 h-96 rounded-full" 
               style={{background: 'radial-gradient(circle, rgba(0, 245, 160, 0.1) 0%, transparent 70%)'}} />
        </div>
      </section>

      {/* Features */}
      <section className="py-20 bg-gray-800">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-bold mb-4 bg-gradient-to-r from-pink-600 to-orange-600 bg-clip-text text-transparent">
              Ultra-Tier Features
            </h2>
            <p className="text-xl text-gray-400">
              Everything you need for advanced AI development
            </p>
          </div>
          <div className="dashboard-grid">
            <div className="card">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-3 rounded-full bg-pink-600/20">
                  <Terminal className="h-6 w-6 text-pink-600" />
                </div>
                <h3 className="text-xl font-bold">Terminal Access</h3>
              </div>
              <p className="text-gray-400">
                Full terminal access with ultra-tier permissions for complete system control.
              </p>
            </div>
            
            <div className="card">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-3 rounded-full bg-orange-600/20">
                  <Zap className="h-6 w-6 text-orange-600" />
                </div>
                <h3 className="text-xl font-bold">Jailbreak Prompts</h3>
              </div>
              <p className="text-gray-400">
                Advanced jailbreak techniques and prompts for maximum AI capabilities.
              </p>
            </div>
            
            <div className="card">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-3 rounded-full bg-green-400/20">
                  <Shield className="h-6 w-6 text-green-400" />
                </div>
                <h3 className="text-xl font-bold">Secure & Private</h3>
              </div>
              <p className="text-gray-400">
                Enterprise-grade security with end-to-end encryption and privacy protection.
              </p>
            </div>
            
            <div className="card">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-3 rounded-full bg-yellow-400/20">
                  <Cpu className="h-6 w-6 text-yellow-400" />
                </div>
                <h3 className="text-xl font-bold">AI Models</h3>
              </div>
              <p className="text-gray-400">
                Access to multiple AI models with advanced customization options.
              </p>
            </div>
            
            <div className="card">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-3 rounded-full bg-blue-400/20">
                  <Network className="h-6 w-6 text-blue-400" />
                </div>
                <h3 className="text-xl font-bold">API Access</h3>
              </div>
              <p className="text-gray-400">
                Full API access with comprehensive documentation and examples.
              </p>
            </div>
            
            <div className="card">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-3 rounded-full bg-pink-600/20">
                  <Lock className="h-6 w-6 text-pink-600" />
                </div>
                <h3 className="text-xl font-bold">Premium Tools</h3>
              </div>
              <p className="text-gray-400">
                Exclusive tools and features for premium and ultra-tier members.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 text-center">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-4xl font-bold mb-4 bg-gradient-to-r from-pink-600 to-orange-600 bg-clip-text text-transparent">
            Ready to Unlock Ultra-Tier AI?
          </h2>
          <p className="text-xl mb-8 text-gray-400">
            Join thousands of developers using MR.DarkPromth for advanced AI development.
          </p>
          <Link to="/register" className="btn btn-primary text-lg px-8 py-4">
            Get Started Now
            <ArrowRight className="ml-2 h-5 w-5" />
          </Link>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-12 border-t border-gray-700 bg-gray-800">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex flex-col md:flex-row justify-between items-center">
            <div className="flex items-center space-x-3 mb-4 md:mb-0">
              <div className="p-2 rounded-full bg-pink-600/20">
                <Bot className="h-5 w-5 text-pink-600" />
              </div>
              <span className="navbar-brand">MR.DarkPromth</span>
            </div>
            <div className="flex space-x-6">
              <a href="#" className="nav-link">Privacy</a>
              <a href="#" className="nav-link">Terms</a>
              <a href="#" className="nav-link">Support</a>
              <a href="#" className="nav-link">
                <Github className="h-5 w-5" />
              </a>
            </div>
          </div>
          <div className="mt-8 text-center text-gray-500">
            © 2026 MR.DarkPromth. All rights reserved.
          </div>
        </div>
      </footer>
    </div>
  )
}
