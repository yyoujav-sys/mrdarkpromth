import React, { useState, useEffect } from 'react'
import { CreditCard, QrCode, Upload, CheckCircle, AlertCircle, Loader } from 'lucide-react'
import { apiClient } from '../lib/api'

type BillingStep = 'select-plan' | 'payment-method' | 'qr-payment' | 'slip-verification' | 'success'

interface Plan {
  id: string
  name: string
  tier: 'premium' | 'ultra'
  price: number
  features: string[]
  duration: string
}

interface QRPaymentData {
  qr_code: string
  payment_id: string
  amount: number
  reference: string
  expires_at: string
}

export const Billing: React.FC = () => {
  const [step, setStep] = useState<BillingStep>('select-plan')
  const [plans, setPlans] = useState<Plan[]>([])
  const [selectedPlan, setSelectedPlan] = useState<Plan | null>(null)
  const [paymentMethod, setPaymentMethod] = useState<'card' | 'qr-code'>('qr-code')
  const [qrData, setQrData] = useState<QRPaymentData | null>(null)
  const [slipFile, setSlipFile] = useState<File | null>(null)
  const [slipPreview, setSlipPreview] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')

  // Fetch plans from backend on component mount
  useEffect(() => {
    const fetchPlans = async () => {
      try {
        setLoading(true)
        const response = await apiClient.getPlans()
        // Transform backend plans to frontend format
        const transformedPlans: Plan[] = response.map((plan: any) => ({
          id: plan.id,
          name: plan.name,
          tier: plan.tier.toLowerCase() as 'premium' | 'ultra',
          price: parseFloat(plan.price),
          features: Array.isArray(plan.features) ? plan.features : [],
          duration: 'month',
        }))
        setPlans(transformedPlans)
      } catch (err: any) {
        setError(err.response?.data?.message || 'Failed to load plans')
      } finally {
        setLoading(false)
      }
    }
    fetchPlans()
  }, [])

  const handleSelectPlan = (plan: Plan) => {
    setSelectedPlan(plan)
    setStep('payment-method')
  }

  const handlePaymentMethodSelect = async () => {
    if (!selectedPlan) return
    if (paymentMethod === 'qr-code') {
      try {
        setLoading(true)
        const response = await apiClient.request({
          method: 'POST',
          url: '/api/billing/generate-qr',
          data: {
            plan_id: selectedPlan.id,
          },
        })
        setQrData(response)
        setStep('qr-payment')
      } catch (err: any) {
        setError(err.response?.data?.message || 'Failed to generate QR code')
      } finally {
        setLoading(false)
      }
    } else {
      // Handle card payment
      setStep('payment-method')
    }
  }

  const handleSlipUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      if (file.size > 5 * 1024 * 1024) {
        setError('File size must be less than 5MB')
        return
      }
      setSlipFile(file)
      const reader = new FileReader()
      reader.onloadend = () => {
        setSlipPreview(reader.result as string)
      }
      reader.readAsDataURL(file)
    }
  }

  const handleVerifySlip = async () => {
    if (!slipFile || !qrData) {
      setError('Please upload a payment slip')
      return
    }
    try {
      setLoading(true)
      // Convert file to base64 for upload
      const toBase64 = (file: File): Promise<string> => new Promise((resolve, reject) => {
        const reader = new FileReader()
        reader.readAsDataURL(file)
        reader.onload = () => resolve(reader.result as string)
        reader.onerror = reject
      })
      const slipBase64 = await toBase64(slipFile)
      const response = await apiClient.request({
        method: 'POST',
        url: '/api/billing/verify-slip',
        data: {
          reference: qrData.reference,
          slip_image: slipBase64
        }
      })
      if (response.verified) {
        setStep('success')
        setSuccess('Payment verified successfully!')
      } else {
        setError('Payment verification failed. Please try again.')
      }
    } catch (err: any) {
      setError(err.response?.data?.message || 'Failed to verify payment slip')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="max-w-6xl mx-auto space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-white">Billing & Subscription</h1>
        <p className="text-gray-400 mt-1">Upgrade your plan to unlock more features</p>
      </div>

      {error && (
        <div className="p-4 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 flex items-start gap-3">
          <AlertCircle className="h-5 w-5 flex-shrink-0 mt-0.5" />
          <span>{error}</span>
        </div>
      )}

      {success && (
        <div className="p-4 bg-green-500/10 border border-green-500/30 rounded-lg text-green-400 flex items-start gap-3">
          <CheckCircle className="h-5 w-5 flex-shrink-0 mt-0.5" />
          <span>{success}</span>
        </div>
      )}

      {step === 'select-plan' && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {plans.map((plan) => (
            <div
              key={plan.id}
              className={`bg-dark-secondary rounded-lg border p-6 cursor-pointer transition-all hover:border-neon-purple/50 ${selectedPlan?.id === plan.id ? 'border-neon-purple ring-1 ring-neon-purple' : 'border-dark-accent'
                }`}
              onClick={() => handleSelectPlan(plan)}
            >
              <h3 className="text-lg font-bold text-white mb-2">{plan.name}</h3>
              <p className="text-3xl font-bold text-neon-purple mb-4">
                ${plan.price}
                <span className="text-sm text-gray-400">/{plan.duration}</span>
              </p>
              <ul className="space-y-2 mb-6">
                {plan.features.map((feature, idx) => (
                  <li key={idx} className="text-sm text-gray-300 flex items-start gap-2">
                    <CheckCircle className="h-4 w-4 text-green-400 flex-shrink-0 mt-0.5" />
                    {feature}
                  </li>
                ))}
              </ul>
              <button
                onClick={() => handleSelectPlan(plan)}
                className="w-full px-4 py-2 bg-neon-purple hover:bg-neon-purple/80 text-white rounded-lg transition-colors font-medium"
              >
                {selectedPlan?.id === plan.id ? 'Selected' : 'Select'}
              </button>
            </div>
          ))}
        </div>
      )}

      {step === 'payment-method' && selectedPlan && (
        <div className="bg-dark-secondary rounded-lg border border-dark-accent p-8">
          <h2 className="text-2xl font-bold text-white mb-6">Select Payment Method</h2>
          <div className="space-y-4 mb-6">
            <label className="flex items-center p-4 border-2 rounded-lg cursor-pointer transition-colors"
              style={{
                borderColor: paymentMethod === 'qr-code' ? '#a78bfa' : '#374151',
                backgroundColor: paymentMethod === 'qr-code' ? 'rgba(167, 139, 250, 0.1)' : 'transparent'
              }}
            >
              <input
                type="radio"
                name="payment"
                value="qr-code"
                checked={paymentMethod === 'qr-code'}
                onChange={(e) => setPaymentMethod(e.target.value as 'qr-code')}
                className="w-4 h-4"
              />
              <div className="ml-4 flex-1">
                <p className="font-medium text-white flex items-center gap-2">
                  <QrCode className="h-5 w-5" />
                  QR Code Payment (G Wallet)
                </p>
                <p className="text-sm text-gray-400">Scan QR code to pay via G Wallet</p>
              </div>
            </label>
            <label className="flex items-center p-4 border-2 rounded-lg cursor-pointer transition-colors opacity-50"
              style={{
                borderColor: paymentMethod === 'card' ? '#a78bfa' : '#374151',
                backgroundColor: paymentMethod === 'card' ? 'rgba(167, 139, 250, 0.1)' : 'transparent'
              }}
            >
              <input
                type="radio"
                name="payment"
                value="card"
                disabled
                checked={paymentMethod === 'card'}
                onChange={(e) => setPaymentMethod(e.target.value as 'card')}
                className="w-4 h-4"
              />
              <div className="ml-4 flex-1">
                <p className="font-medium text-white flex items-center gap-2">
                  <CreditCard className="h-5 w-5" />
                  Credit/Debit Card
                </p>
                <p className="text-sm text-gray-400">Coming soon</p>
              </div>
            </label>
          </div>
          <div className="flex gap-4">
            <button
              onClick={() => setStep('select-plan')}
              className="flex-1 px-4 py-2 bg-dark-accent hover:bg-dark-accent/80 text-gray-300 rounded-lg transition-colors font-medium"
            >
              Back
            </button>
            <button
              onClick={handlePaymentMethodSelect}
              disabled={loading || paymentMethod === 'card'}
              className="flex-1 px-4 py-2 bg-neon-purple hover:bg-neon-purple/80 disabled:bg-gray-600 text-white rounded-lg transition-colors font-medium flex items-center justify-center gap-2"
            >
              {loading && <Loader className="h-4 w-4 animate-spin" />}
              {loading ? 'Generating...' : 'Continue'}
            </button>
          </div>
        </div>
      )}

      {step === 'qr-payment' && qrData && selectedPlan && (
        <div className="bg-dark-secondary rounded-lg border border-dark-accent p-8">
          <h2 className="text-2xl font-bold text-white mb-6">Scan to Pay</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="flex flex-col items-center justify-center bg-white p-4 rounded-lg">
              <img src={qrData.qr_code} alt="Payment QR Code" className="w-64 h-64" />
              <p className="text-dark-primary font-bold mt-2">Scan with G Wallet</p>
            </div>
            <div className="space-y-6">
              <div className="bg-dark-primary/50 rounded-lg p-4 border border-dark-accent">
                <h3 className="text-sm font-medium text-gray-400 uppercase mb-3">Payment Details</h3>
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-gray-400">Plan:</span>
                    <span className="text-white font-medium">{selectedPlan.name}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-400">Amount:</span>
                    <span className="text-white font-medium">${qrData.amount}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-400">Reference:</span>
                    <span className="text-white font-mono text-sm">{qrData.reference}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-400">Expires:</span>
                    <span className="text-white font-medium">
                      {new Date(qrData.expires_at).toLocaleTimeString()}
                    </span>
                  </div>
                </div>
              </div>
              <div>
                <h3 className="text-lg font-bold text-white mb-4">Next Steps</h3>
                <ol className="space-y-2 text-gray-300 text-sm">
                  <li>1. Open your G Wallet app</li>
                  <li>2. Tap the QR scanner button</li>
                  <li>3. Scan the QR code on the left</li>
                  <li>4. Confirm the payment amount</li>
                  <li>5. Take a screenshot of the payment slip</li>
                  <li>6. Upload the slip below to verify</li>
                </ol>
              </div>
              <button
                onClick={() => setStep('slip-verification')}
                className="w-full px-4 py-2 bg-neon-purple hover:bg-neon-purple/80 text-white rounded-lg transition-colors font-medium"
              >
                I've Completed Payment
              </button>
            </div>
          </div>
        </div>
      )}

      {step === 'slip-verification' && selectedPlan && (
        <div className="bg-dark-secondary rounded-lg border border-dark-accent p-8">
          <h2 className="text-2xl font-bold text-white mb-6">Upload Payment Slip</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div>
              <label className="flex flex-col items-center justify-center w-full h-64 border-2 border-dashed border-dark-accent rounded-lg cursor-pointer hover:border-neon-purple/50 transition-colors bg-dark-primary/50">
                <div className="flex flex-col items-center justify-center pt-5 pb-6">
                  <Upload className="h-8 w-8 text-gray-400 mb-2" />
                  <p className="text-sm text-gray-400">Click to upload payment slip</p>
                  <p className="text-xs text-gray-500">PNG, JPG, PDF up to 5MB</p>
                </div>
                <input
                  type="file"
                  accept="image/*,.pdf"
                  onChange={handleSlipUpload}
                  className="hidden"
                />
              </label>
            </div>
            {slipPreview && (
              <div>
                <p className="text-gray-400 text-sm mb-2">Preview:</p>
                <img src={slipPreview} alt="Slip Preview" className="w-full h-64 object-contain rounded-lg border border-dark-accent" />
              </div>
            )}
          </div>
          <div className="mt-6 flex gap-4">
            <button
              onClick={() => setStep('qr-payment')}
              className="flex-1 px-4 py-2 bg-dark-accent hover:bg-dark-accent/80 text-gray-300 rounded-lg transition-colors font-medium"
            >
              Back
            </button>
            <button
              onClick={handleVerifySlip}
              disabled={loading || !slipFile}
              className="flex-1 px-4 py-2 bg-neon-purple hover:bg-neon-purple/80 disabled:bg-gray-600 text-white rounded-lg transition-colors font-medium flex items-center justify-center gap-2"
            >
              {loading && <Loader className="h-4 w-4 animate-spin" />}
              {loading ? 'Verifying...' : 'Verify Payment'}
            </button>
          </div>
        </div>
      )}

      {step === 'success' && selectedPlan && (
        <div className="bg-dark-secondary rounded-lg border border-dark-accent p-8 text-center">
          <CheckCircle className="h-16 w-16 text-green-500 mx-auto mb-4" />
          <h2 className="text-2xl font-bold text-white mb-2">Payment Successful!</h2>
          <p className="text-gray-400 mb-6">
            Your subscription to {selectedPlan.name} has been activated
          </p>
          <button
            onClick={() => window.location.href = '/app/profile'}
            className="px-6 py-2 bg-neon-purple hover:bg-neon-purple/80 text-white rounded-lg transition-colors font-medium"
          >
            Go to Profile
          </button>
        </div>
      )}
    </div>
  )
}

export default Billing
