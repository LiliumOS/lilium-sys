
/// The base subsystem is
#define SUBSYSTEM_BASE (0)

#define SYS_ShareHandle (0)
#define SYS_UnshareHandle (1)
#define SYS_UpgradeSharedHandle (2)
#define SYS_IdentHandle (3)
#define SYS_CheckHandleRight (4)
#define SYS_DropHandleRight (5)
#define SYS_DropAllHandleRights (6)
#define SYS_GrantHandleRight (7)

#define SYS_CreateSecurityContext (16)

#define SYS_CopySecurityContext (17)

#define SYS_DestroySecurityContext (18)

#define SYS_GetCurrentSecurityContext (19)

#define SYS_HasKernelPermission (20)

#define SYS_HasThreadPermission (21)

#define SYS_HasProcessPermission (22)

#define SYS_SetPrimaryPrinciple (23)

#define SYS_AddSecondaryPrincipal (24)

#define SYS_GrantKernelPermission (25)

#define SYS_GrantThreadPermission (26)

#define SYS_GrantProcessPermission (27)

#define SYS_DropKernelPermission (28)

#define SYS_DropThreadPermission (29)

#define SYS_DropProcessPermission (30)

#define SYS_RevokeKernelPermission (31)

#define SYS_RevokeThreadPermission (32)

#define SYS_RevokeProcessPermission (33)

#define SYS_SetKernelResourceLimit (34)

#define SYS_GetKernelResourceLimit (34)

#define SYS_EncodeSecurityContext (34)

#define SYS_GetPrimaryPrincipal (35)

#define SYS_GetSecondaryPrincipals (36)

#define SYS_GetSystemInfo (48)

#define SYS_GetProcessorInfo (49)

#define SYS_SetArchConfig (50)

#define SYS_GetProvidedArchConfig (51)

#define SYS_GetActiveArchConfig (52)

#define SYS_UnmanagedException (64)

#define SYS_ExceptInstallHandler (65)

#define SYS_ExceptHandleSynchronous (66)

#define SYS_ExceptRaiseAsynchronous (67)

#define SYS_ExceptResumeAt (68)

#define SYS_ExceptSetGPR (69)

#define SYS_ExceptSetPointerReg (70)

#define SYS_ExceptSetRegister (71)

#define SYS_ExceptGetStopAddr (72)

#define SYS_ExceptGetRegister (73)